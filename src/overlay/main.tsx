import React from "react";
import ReactDOM from "react-dom/client";
import { OverlayApp, initOverlayWindow } from "./OverlayApp";

initOverlayWindow().catch(console.error);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <OverlayApp />
  </React.StrictMode>,
);
