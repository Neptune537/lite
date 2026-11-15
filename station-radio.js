"use strict";

const stationForm = document.getElementById("station-radio");
const stationUrl = document.getElementById("station-url");
const stationAudio = document.getElementById("station-audio");
const stationStatus = document.getElementById("station-status");

const isLoopback = ["localhost", "127.0.0.1", "[::1]"].includes(location.hostname);
if (isLoopback) {
  stationUrl.value = "http://127.0.0.1:8080/live.mp3";
} else {
  stationUrl.value = "https://radio.tr7n7t7.com/live.mp3";
}

stationForm.addEventListener("submit", (event) => {
  event.preventDefault();
  const url = new URL(stationUrl.value);
  const loopback = ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname);
  if (url.username || url.password || (url.protocol !== "https:" &&
      !(url.protocol === "http:" && (loopback || location.protocol === "http:")))) {
    stationStatus.textContent = "Use an HTTPS stream or a local HTTP stream.";
    return;
  }
  stationAudio.src = url.href;
  stationStatus.textContent = "Connecting to live radio…";
  stationAudio.play().catch((error) => {
    if (error.name !== "AbortError") stationStatus.textContent = "Live stream unavailable.";
  });
});
stationAudio.addEventListener("playing", () => {
  stationStatus.textContent = "Playing live radio";
});
stationAudio.addEventListener("waiting", () => {
  stationStatus.textContent = "Buffering live radio…";
});
async function pollRadioStatus() {
  try {
    const statusApiUrl = isLoopback ? "http://127.0.0.1:8080/api/status" : "https://radio.tr7n7t7.com/api/status";
    const res = await fetch(statusApiUrl, { cache: "no-store" });
    if (res.ok) {
      const data = await res.json();
      if (data && data.now_playing) {
        const title = data.now_playing.title || "Unknown Track";
        const host = data.now_playing.speaker || (data.active_daughter && data.active_daughter.name) || "Seven Daughters";
        if (!stationAudio.paused && stationAudio.currentTime > 0) {
          stationStatus.textContent = `📻 [${host}] ${title}`;
        }
      }
    }
  } catch (_) {}
}

setInterval(pollRadioStatus, 10000);
pollRadioStatus();

stationAudio.addEventListener("error", () => {
  if (isLoopback && stationAudio.src.includes(":8080/")) {
    stationUrl.value = "http://127.0.0.1:8082/live.mp3";
    stationAudio.src = stationUrl.value;
    stationStatus.textContent = "Retrying local fallback (8082)…";
    stationAudio.play().catch(() => {
      stationStatus.textContent = "Live stream unavailable.";
    });
    return;
  }
  stationStatus.textContent = "Live stream unavailable.";
});
