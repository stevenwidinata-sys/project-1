import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface User {
  id: number;
  display_name: String;
  email?: string;
  role: string;
}

export default function App() {
  const [tab, setTab] = useState<"passenger" | "employee">("passenger");
  const [mode, setMode] = useState<"login" | "register" | "forgot">("login");

  // Form states
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [employeeCode, setEmployeeCode] = useState("");

  // Feedback states
  const [statusMsg, setStatusMsg] = useState<{ text: string; error: boolean } | null>(null);
  const [currentUser, setCurrentUser] = useState<User | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setStatusMsg(null);

    try {
      if (mode === "forgot") {
        const res = await invoke<string>("forgot_password", { email });
        setStatusMsg({ text: res, error: false });
        return;
      }

      if (tab === "passenger") {
        if (mode === "register") {
          const res = await invoke<string>("register_passenger", {
            displayName,
            email,
            password,
          });
          setStatusMsg({ text: res, error: false });
          setMode("login");
        } else {
          const user = await invoke<User>("login_passenger", { email, password });
          setCurrentUser(user);
        }
      } else {
        const user = await invoke<User>("login_employee", {
          employeeCode,
          password,
        });
        setCurrentUser(user);
      }
    } catch (err: any) {
      setStatusMsg({ text: err.toString(), error: true });
    }
  };

  if (currentUser) {
    return (
      <div className="auth-container">
        <div className="auth-card welcome-card">
          <h2>Welcome, {currentUser.display_name}!</h2>
          <p className="role-badge">Role: <span>{currentUser.role}</span></p>
          <button className="btn btn-secondary" onClick={() => setCurrentUser(null)}>
            Sign Out
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="auth-container">
      <div className="auth-card">
        <h1 className="brand-title">SailLantis</h1>
        
        {/* User Type Tabs */}
        <div className="tab-group">
          <button
            className={`tab-btn ${tab === "passenger" ? "active" : ""}`}
            onClick={() => { setTab("passenger"); setMode("login"); }}
          >
            Passenger
          </button>
          <button
            className={`tab-btn ${tab === "employee" ? "active" : ""}`}
            onClick={() => { setTab("employee"); setMode("login"); }}
          >
            Staff / Employee
          </button>
        </div>

        {/* Sub-Header */}
        <h3 className="form-title">
          {mode === "forgot"
            ? "Reset Password"
            : `${tab === "passenger" ? "Passenger" : "Employee"} ${mode === "login" ? "Login" : "Registration"}`}
        </h3>

        {/* Feedback Message */}
        {statusMsg && (
          <div className={`status-box ${statusMsg.error ? "error" : "success"}`}>
            {statusMsg.text}
          </div>
        )}

        <form onSubmit={handleSubmit} className="auth-form">
          {tab === "passenger" && mode === "register" && (
            <div className="form-field">
              <label>Display Name</label>
              <input
                type="text"
                required
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                placeholder="John Doe"
              />
            </div>
          )}

          {tab === "passenger" ? (
            <div className="form-field">
              <label>Email Address</label>
              <input
                type="email"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="name@example.com"
              />
            </div>
          ) : (
            <div className="form-field">
              <label>Employee Code</label>
              <input
                type="text"
                required
                value={employeeCode}
                onChange={(e) => setEmployeeCode(e.target.value)}
                placeholder="EMP-1002"
              />
            </div>
          )}

          {mode !== "forgot" && (
            <div className="form-field">
              <label>Password</label>
              <input
                type="password"
                required
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
              />
            </div>
          )}

          <button type="submit" className="btn btn-primary">
            {mode === "forgot" ? "Send Reset Link" : mode === "login" ? "Sign In" : "Create Account"}
          </button>
        </form>

        {/* Footer Navigation Links */}
        <div className="auth-footer">
          {tab === "passenger" && (
            <p>
              {mode === "login" ? "Don't have an account? " : "Already registered? "}
              <button
                type="button"
                className="link-btn"
                onClick={() => setMode(mode === "login" ? "register" : "login")}
              >
                {mode === "login" ? "Sign up" : "Log in"}
              </button>
            </p>
          )}

          {mode === "login" && (
            <button
              type="button"
              className="link-btn forgot-btn"
              onClick={() => setMode("forgot")}
            >
              Forgot Password?
            </button>
          )}

          {mode === "forgot" && (
            <button
              type="button"
              className="link-btn"
              onClick={() => setMode("login")}
            >
              Back to Login
            </button>
          )}
        </div>
      </div>
    </div>
  );
}