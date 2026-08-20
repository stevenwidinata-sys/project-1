import React from "react";
import { invoke } from "@tauri-apps/api/tauri";

export default function Home() {
  const callGreet = async () => {
    try {
      const response = await invoke("greet", { name: "Frontend User" });
      alert(response);
    } catch (e) {
      console.error(e);
      alert("Failed to call greet");
    }
  };

  return (
    <div style={{ padding: 40, fontFamily: 'Arial, sans-serif' }}>
      <header style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <img src="/logo.svg" alt="saILintis" width={48} height={48} />
        <h1>saILintis - Desktop App (Frontend Scaffold)</h1>
      </header>
      <main style={{ marginTop: 24 }}>
        <p>Welcome to the saILintis Next.js frontend scaffold. Click to call a Rust command via Tauri.</p>
        <button onClick={callGreet} style={{ padding: '8px 12px', fontSize: 16 }}>Call greet()</button>
      </main>
    </div>
  );
}
