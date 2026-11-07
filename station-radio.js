"use strict";

const stationForm = document.getElementById("station-radio");
const stationUrl = document.getElementById("station-url");
const stationAudio = document.getElementById("station-audio");
const stationStatus = document.getElementById("station-status");

if (["localhost", "127.0.0.1", "[::1]"].includes(location.hostname)) {
  stationUrl.value = "http://127.0.0.1:8082/live.mp3";
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
stationAudio.addEventListener("pause", () => {
  stationStatus.textContent = "Radio paused";
});
stationAudio.addEventListener("error", () => {
  stationStatus.textContent = "Live stream unavailable.";
});
