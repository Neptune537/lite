"use strict";

const SONAR_TOPICS = ["gyre:sonar:tactical", "gyre:telemetry:gods_eye"];
const SONAR_TTL_MS = 30000;

function decodeHex(value) {
  if (typeof value !== "string" || value.length % 2 || !/^[0-9a-f]+$/i.test(value)) {
    throw new Error("Invalid signing material");
  }
  return Uint8Array.from(value.match(/../g), byte => parseInt(byte, 16));
}

function validateSonarEndpoint(address) {
  const endpoint = new URL(address);
  const localPage = ["localhost", "127.0.0.1", "[::1]"].includes(location.hostname);
  const loopback = ["localhost", "127.0.0.1", "[::1]"].includes(endpoint.hostname);
  if ((endpoint.protocol !== "wss:" && !(localPage && loopback && endpoint.protocol === "ws:")) ||
      endpoint.pathname !== "/ws/sonar" || endpoint.search || endpoint.hash ||
      endpoint.username || endpoint.password) {
    throw new Error("Use /ws/sonar over WSS. Loopback WS requires a local development page.");
  }
  return endpoint.href;
}

function connectHeavenlySonar(addresses, token, publisherKeys, onState, publicReadOnly = false) {
  const endpoints = addresses.map(validateSonarEndpoint);
  if (!endpoints.length) throw new Error("No telemetry endpoint configured");
  if (!publicReadOnly && !/^[0-9a-f]{64}$/i.test(token)) throw new Error("Enter a read-only Sonar token.");
  let socket, retry, timeout;
  let endpointIndex = 0;
  let delay = 500;
  let stopped = false;
  let lastValidAt = 0;
  const keys = new Map();
  const expireTimer = setInterval(() => {
    window.expireHeavenlyTelemetry?.(Date.now());
    if (lastValidAt && Date.now() - lastValidAt >= SONAR_TTL_MS) {
      lastValidAt = 0;
      onState("Feed unavailable · no recent verified telemetry");
    }
  }, 1000);

  function open() {
    if (stopped) return;
    onState("Connecting to telemetry");
    const protocols = publicReadOnly ? ["sonar.v1"] : ["sonar.v1", "sonar.token." + token];
    const connection = new WebSocket(endpoints[endpointIndex], protocols);
    socket = connection;
    function reconnect() {
      if (stopped || socket !== connection) return;
      clearTimeout(timeout);
      socket = undefined;
      connection.close();
      endpointIndex = (endpointIndex + 1) % endpoints.length;
      onState("Feed unavailable · reconnecting");
      retry = setTimeout(open, delay);
      delay = Math.min(delay * 2, 30000);
    }
    timeout = setTimeout(reconnect, 5000);
    connection.onopen = () => {
      if (stopped || socket !== connection) return;
      clearTimeout(timeout);
      timeout = setTimeout(reconnect, SONAR_TTL_MS);
      for (const topic of SONAR_TOPICS) connection.send(JSON.stringify({action: "subscribe", topic}));
      onState("Transport connected · awaiting verified telemetry");
    };
    let pendingMessages = 0;
    let messageChain = Promise.resolve();
    connection.onmessage = ({data}) => {
      if (stopped || socket !== connection || typeof data !== "string" || data.length > 65536) return;
      if (pendingMessages >= 128) {
        reconnect();
        return;
      }
      pendingMessages += 1;
      messageChain = messageChain.then(async () => {
        try {
          if (stopped || socket !== connection) return;
          const frame = JSON.parse(data);
          if (frame.action && (frame.action !== "publish" || !SONAR_TOPICS.includes(frame.topic))) return;
          const event = frame.action ? frame.event : frame;
          if (event.eventType !== "MESH_SIGNAL" || typeof event.payload?.message !== "string") return;
          const keyHex = publisherKeys[event.did];
          if (typeof keyHex !== "string" || keyHex.length !== 64) throw new Error("Untrusted publisher");
          let key = keys.get(event.did);
          if (!key) {
            key = await crypto.subtle.importKey("raw", decodeHex(keyHex), {name: "Ed25519"}, false, ["verify"]);
            keys.set(event.did, key);
          }
          const message = new TextEncoder().encode(event.payload.message);
          if (typeof event.signature !== "string" || event.signature.length !== 128 ||
              !await crypto.subtle.verify("Ed25519", key, decodeHex(event.signature), message)) {
            throw new Error("Invalid telemetry signature");
          }
          const update = JSON.parse(event.payload.message);
          const now = Date.now();
          if (update.source_did !== event.did || !Number.isSafeInteger(update.observed_at_unix_ms) ||
              now - update.observed_at_unix_ms >= SONAR_TTL_MS ||
              update.observed_at_unix_ms > now + 5000 ||
              !Number.isSafeInteger(event.timestamp) ||
              Math.abs(event.timestamp * 1000 - update.observed_at_unix_ms) > 1000 ||
              typeof update.entity?.entity_id !== "string") throw new Error("Invalid telemetry timestamp");
          if (stopped || socket !== connection) return;
          update.entity.entity_id = event.did + "/" + update.entity.entity_id;
          const json = JSON.stringify(update);
          const accepted = window.ingestHeavenlyTelemetry
            ? window.ingestHeavenlyTelemetry(json, now)
            : window.wasmBindings?.ingest_telemetry_update?.(json, now);
          if (accepted) {
            lastValidAt = Math.max(lastValidAt, update.observed_at_unix_ms);
            clearTimeout(timeout);
            timeout = setTimeout(reconnect, Math.max(1, SONAR_TTL_MS - (now - lastValidAt)));
            delay = 500;
            onState("Receiving signed live telemetry");
          }
        } catch (error) {
          if (!stopped && socket === connection) onState("Telemetry rejected · " + error.message);
        } finally {
          pendingMessages -= 1;
        }
      });
      return messageChain;
    };
    connection.onerror = reconnect;
    connection.onclose = reconnect;
  }
  open();
  return () => {
    stopped = true;
    clearTimeout(retry);
    clearTimeout(timeout);
    clearInterval(expireTimer);
    socket?.close();
    onState("Disconnected · tracks expire within 30 seconds");
  };
}

async function initializeSonar() {
  const form = document.getElementById("sonar-connection");
  if (!form) return;
  const state = document.getElementById("sonar-connection-state");
  const tokenInput = document.getElementById("sonar-token");
  const button = form.querySelector("button");
  let config = {};
  let disconnect;
  let manuallySelected = false;
  const onState = value => { state.textContent = value; };
  form.addEventListener("submit", event => {
    event.preventDefault();
    manuallySelected = true;
    if (disconnect) {
      disconnect();
      disconnect = undefined;
      button.textContent = "Connect";
      return;
    }
    try {
      const token = tokenInput.value;
      const address = form.elements.endpoint.value;
      const publicReadOnly = !token && config.public_read_only === true && config.endpoints?.includes(address);
      tokenInput.value = "";
      disconnect = connectHeavenlySonar([address], token, config.publisher_keys || {}, onState, publicReadOnly);
      button.textContent = "Disconnect";
    } catch (error) { onState(error.message); }
  });
  button.disabled = false;
  try {
    const response = await fetch("sonar-config.json", {cache: "no-store"});
    if (!response.ok) throw new Error("Telemetry configuration unavailable");
    config = await response.json();
    if (manuallySelected) return;
    const endpoints = config.endpoints || [];
    if (endpoints.length) {
      form.elements.endpoint.value = endpoints[0];
      disconnect = connectHeavenlySonar(endpoints, undefined, config.publisher_keys || {}, onState, config.public_read_only === true);
      button.textContent = "Disconnect";
    } else {
      onState("Feed unavailable · no relay configured");
    }
  } catch (error) { onState(error.message); }
}
if (typeof document !== "undefined") initializeSonar();
