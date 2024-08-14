const socket = new WebSocket("ws://localhost:8080/ws/");

socket.addEventListener("close", () => {
    setTimeout(() => window.location.reload(true), 1000);
});
