"use client";

import React, { useState, useEffect, useRef } from "react";
import { Download, Github, Keyboard, Volume2, VolumeX, Shield, Zap, Info, ChevronDown, ChevronRight, CornerDownLeft, Star } from "lucide-react";

// Types for the mixer simulation
interface AudioSession {
  id: string;
  name: string;
  volume: number;
  prevVolume?: number; // Store for muting
  isMuted: boolean;
  isDiscord?: boolean;
}

interface DiscordUser {
  id: string;
  username: string;
  volume: number;
  prevVolume?: number;
  isMuted: boolean;
}

export default function Home() {
  // Simulator State
  const [sessions, setSessions] = useState<AudioSession[]>([
    { id: "master", name: "System Master", volume: 70, isMuted: false },
    { id: "discord", name: "Discord", volume: 85, isMuted: false, isDiscord: true },
    { id: "spotify", name: "Spotify", volume: 45, isMuted: false },
    { id: "chrome", name: "Chrome (YouTube)", volume: 30, isMuted: false },
    { id: "steam", name: "Steam Client", volume: 15, isMuted: false },
  ]);

  const [discordUsers, setDiscordUsers] = useState<DiscordUser[]>([
    { id: "user1", username: "Bob", volume: 100, isMuted: false },
    { id: "user2", username: "Alice (Speaking)", volume: 120, isMuted: false },
    { id: "user3", username: "Charlie", volume: 80, isMuted: true },
  ]);

  const [selectedIndex, setSelectedIndex] = useState<number>(0);
  const [selectedDiscordIndex, setSelectedDiscordIndex] = useState<number>(0);
  const [isDiscordExpanded, setIsDiscordExpanded] = useState<boolean>(false);
  const [isFocused, setIsFocused] = useState<boolean>(false);
  const [lastKeyPressed, setLastKeyPressed] = useState<string>("");
  const [isMobile, setIsMobile] = useState<boolean>(false);
  const [stars, setStars] = useState<number | null>(null);

  const simulatorRef = useRef<HTMLDivElement>(null);
  const listContainerRef = useRef<HTMLDivElement>(null);

  // Fetch GitHub stars on load
  useEffect(() => {
    fetch("https://api.github.com/repos/Starz099/oto")
      .then((res) => {
        if (!res.ok) throw new Error("Failed to fetch stars");
        return res.json();
      })
      .then((data) => {
        if (typeof data.stargazers_count === "number") {
          setStars(data.stargazers_count);
        }
      })
      .catch((err) => {
        console.error("Error loading GitHub stars:", err);
      });
  }, []);

  const formattedStars = (() => {
    if (stars === null) return "";
    if (stars < 1000) return stars.toString();
    const formatted = (stars / 1000).toFixed(1);
    return formatted.endsWith(".0")
      ? `${formatted.slice(0, -2)}k`
      : `${formatted}k`;
  })();

  // Check if mobile
  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768);
    };
    checkMobile();
    window.addEventListener("resize", checkMobile);
    return () => window.removeEventListener("resize", checkMobile);
  }, []);

  // Auto-scroll selected element into view
  useEffect(() => {
    if (!isFocused || !listContainerRef.current) return;
    const selectedEl = listContainerRef.current.querySelector("[data-selected='true']");
    if (selectedEl) {
      selectedEl.scrollIntoView({
        block: "nearest",
        behavior: "smooth"
      });
    }
  }, [selectedIndex, selectedDiscordIndex, isDiscordExpanded, isFocused]);

  // Handle Keyboard Navigation inside the Simulator
  useEffect(() => {
    if (!isFocused) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      const key = e.key.toUpperCase();
      let handled = false;

      // Map key binds
      if (e.key === "ArrowDown" || key === "J") {
        setLastKeyPressed(key === "J" ? "J" : "↓");
        handled = true;
        if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
          setSelectedDiscordIndex((prev) => Math.min(prev + 1, discordUsers.length - 1));
        } else {
          setSelectedIndex((prev) => Math.min(prev + 1, sessions.length - 1));
        }
      } else if (e.key === "ArrowUp" || key === "K") {
        setLastKeyPressed(key === "K" ? "K" : "↑");
        handled = true;
        if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
          if (selectedDiscordIndex === 0) {
            // Jump back up to session list
            setIsDiscordExpanded(false);
          } else {
            setSelectedDiscordIndex((prev) => Math.max(prev - 1, 0));
          }
        } else {
          setSelectedIndex((prev) => Math.max(prev - 1, 0));
        }
      } else if (key === "L" || e.key === "ArrowRight") {
        setLastKeyPressed(key === "L" ? "L" : "→");
        handled = true;
        const step = e.shiftKey ? 10 : 2; // Shift modifier

        if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
          setDiscordUsers((users) =>
            users.map((u, i) =>
              i === selectedDiscordIndex
                ? { ...u, volume: Math.min(u.volume + step, 200), isMuted: false }
                : u
            )
          );
        } else {
          setSessions((prevSessions) =>
            prevSessions.map((s, i) =>
              i === selectedIndex
                ? { ...s, volume: Math.min(s.volume + step, 100), isMuted: false }
                : s
            )
          );
        }
      } else if (key === "H" || e.key === "ArrowLeft") {
        setLastKeyPressed(key === "H" ? "H" : "←");
        handled = true;
        const step = e.shiftKey ? 10 : 2;

        if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
          setDiscordUsers((users) =>
            users.map((u, i) =>
              i === selectedDiscordIndex
                ? { ...u, volume: Math.max(u.volume - step, 0) }
                : u
            )
          );
        } else {
          setSessions((prevSessions) =>
            prevSessions.map((s, i) =>
              i === selectedIndex
                ? { ...s, volume: Math.max(s.volume - step, 0) }
                : s
            )
          );
        }
      } else if (key === "M") {
        setLastKeyPressed("M");
        handled = true;
        if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
          setDiscordUsers((users) =>
            users.map((u, i) =>
              i === selectedDiscordIndex
                ? { ...u, isMuted: !u.isMuted }
                : u
            )
          );
        } else {
          setSessions((prevSessions) =>
            prevSessions.map((s, i) =>
              i === selectedIndex
                ? {
                    ...s,
                    isMuted: !s.isMuted,
                    prevVolume: s.volume > 0 ? s.volume : s.prevVolume || 100,
                    volume: s.isMuted ? (s.prevVolume || 100) : 0,
                  }
                : s
            )
          );
        }
      } else if (e.key === "Enter") {
        setLastKeyPressed("ENTER");
        handled = true;
        if (sessions[selectedIndex]?.isDiscord) {
          setIsDiscordExpanded((prev) => !prev);
          setSelectedDiscordIndex(0);
        }
      } else if (e.key === "Escape") {
        setLastKeyPressed("ESC");
        handled = true;
        if (isDiscordExpanded) {
          setIsDiscordExpanded(false);
        } else {
          setIsFocused(false);
        }
      }

      if (handled) {
        e.preventDefault();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isFocused, selectedIndex, selectedDiscordIndex, isDiscordExpanded, sessions, discordUsers]);

  // Click outside listener for the simulator focus
  useEffect(() => {
    const clickOutside = (e: MouseEvent) => {
      if (simulatorRef.current && !simulatorRef.current.contains(e.target as Node)) {
        setIsFocused(false);
      }
    };
    document.addEventListener("mousedown", clickOutside);
    return () => document.removeEventListener("mousedown", clickOutside);
  }, []);

  // UI helpers
  const getVolumeDisplay = (session: AudioSession) => {
    if (session.isMuted || session.volume === 0) return "Muted";
    return `${session.volume}%`;
  };

  return (
    <div className="min-h-screen bg-oto-dark noise-bg flex flex-col justify-between overflow-x-hidden">
      
      {/* HEADER NAVBAR */}
      <header className="max-w-6xl mx-auto w-full px-6 py-6 flex items-center justify-between border-b border-oto-selection/40">
        <div className="flex items-center gap-3">
          <span className="font-display text-2xl font-bold tracking-wide text-oto-pink">oto</span>
        </div>
        <nav className="flex items-center gap-3">
          <a 
            href="https://github.com/Starz099/oto/releases/download/v0.1.2/oto-0.1.2-x86_64.msi"
            className="relative overflow-hidden text-xs font-mono text-oto-dark bg-oto-pink hover:bg-oto-rose hover:text-oto-dark px-3 py-1.5 rounded-md font-bold transition-all"
          >
            <span>Install</span>
            <span className="custom-shine-element" />
          </a>
          <a 
            href="https://github.com/Starz099/oto" 
            target="_blank" 
            rel="noopener noreferrer"
            className="relative overflow-hidden flex items-center gap-2 text-xs font-mono text-oto-rose hover:text-oto-pink transition-colors border border-oto-rose/40 hover:border-oto-pink/80 px-3 py-1.5 rounded-md"
          >
            <Github size={12} className="relative z-10" />
            <span className="relative z-10">GitHub</span>
            {stars !== null && (
              <>
                <span className="relative z-10 h-3.5 w-[1px] bg-oto-rose/30 shrink-0" />
                <span className="relative z-10 text-oto-pink flex items-center gap-1 font-bold shrink-0">
                  <Star size={10} className="fill-oto-pink stroke-none" />
                  <span>{formattedStars}</span>
                </span>
              </>
            )}
            <span className="custom-shine-element" />
          </a>
        </nav>
      </header>

      {/* HERO SECTION */}
      <main className="flex-1 max-w-6xl mx-auto w-full px-6 py-12 md:py-20 flex flex-col lg:flex-row items-center gap-16">
        
        {/* HERO COPY */}
        <section className="flex-1 space-y-8 text-center lg:text-left">
          <div className="space-y-4">
            <h1 className="text-4xl md:text-5xl lg:text-6xl font-bold font-display text-oto-white leading-tight">
              A Keyboard-First <br/>
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-oto-pink to-oto-rose">Audio Mixer Overlay</span>
            </h1>
            <p className="text-md md:text-lg text-oto-slate max-w-xl mx-auto lg:mx-0 font-sans font-light leading-relaxed">
              A lightweight, keyboard-first desktop audio mixer overlay for windows written in Rust, featuring app-specific volume control and zero-latency global push-to-talk, currently supporting Discord.
            </p>
          </div>

          <div className="flex flex-col sm:flex-row items-center justify-center lg:justify-start gap-4">
            <a
              href="https://github.com/Starz099/oto/releases/download/v0.1.2/oto-0.1.2-x86_64.msi"
              className="relative overflow-hidden w-full sm:w-auto px-6 py-4 bg-oto-pink text-oto-dark font-sans font-semibold rounded-lg border border-oto-rose/50 hover:bg-oto-rose hover:text-oto-dark transition-all duration-200 text-center shadow-lg shadow-oto-pink/10 hover:shadow-oto-rose/25 flex items-center justify-center gap-2 group"
            >
              <Download size={18} className="group-hover:translate-y-0.5 transition-transform relative z-10" />
              <span className="relative z-10">Download Now</span>
              <span className="custom-shine-element" />
            </a>
            <a
              href="#features"
              className="relative overflow-hidden w-full sm:w-auto px-6 py-4 bg-[#141414] hover:bg-oto-selection text-oto-white font-sans font-semibold rounded-lg border border-oto-selection transition-all text-center flex items-center justify-center gap-2"
            >
              <span className="relative z-10">Learn More</span>
              <ChevronRight size={16} className="relative z-10" />
              <span className="custom-shine-element" />
            </a>
          </div>

        </section>

        {/* THE INTERACTIVE SIMULATOR (Signature Element) */}
        <section className="flex-1 w-full max-w-lg">
          <div className="space-y-4">
            {/* Simulated Desktop Window */}
            <div 
              ref={simulatorRef}
              onClick={() => setIsFocused(true)}
              className={`w-full bg-[#151518] rounded-xl border transition-all duration-300 relative shadow-2xl overflow-hidden select-none cursor-pointer ${
                isFocused 
                  ? "border-oto-pink shadow-oto-pink/5 scale-[1.01]" 
                  : "border-oto-selection hover:border-oto-slate/30"
              }`}
            >
              {/* Top Window Bar */}
              <div className="px-4 py-3 bg-[#111114] border-b border-oto-selection/30 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-2.5 h-2.5 rounded-full bg-oto-rose/40"></div>
                  <span className="font-mono text-xs text-oto-slate">oto overlay simulator</span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-[10px] font-mono bg-oto-pink/10 text-oto-pink px-2 py-0.5 rounded">PTT READY</span>
                  <span className="text-xs text-oto-slate">⚙</span>
                </div>
              </div>

              {/* App Display Panel */}
              <div ref={listContainerRef} className="p-5 space-y-4 h-[270px] overflow-y-auto scroll-smooth">
                {sessions.map((session, index) => {
                  const isSelected = index === selectedIndex && !isDiscordExpanded;
                  return (
                    <div key={session.id} className="space-y-2">
                      {/* Session Row */}
                      <div
                        data-selected={isSelected}
                        className={`flex items-center justify-between px-3 py-2.5 rounded-lg transition-colors ${
                          isSelected ? "bg-oto-selection text-oto-white" : "bg-transparent text-oto-slate"
                        }`}
                      >
                        <div className="flex items-center gap-3">
                          <div className={`w-1 h-4 rounded ${isSelected ? "bg-oto-pink" : "bg-transparent"}`}></div>
                          <span className={`font-mono text-xs font-medium ${isSelected ? "text-oto-white" : "text-oto-slate"}`}>
                            {session.name}
                          </span>
                        </div>

                        {/* Slider and Text Control */}
                        <div className="flex items-center gap-4">
                          {/* Value Display */}
                          <span className="font-mono text-[11px] text-right w-10 text-oto-rose">
                            {getVolumeDisplay(session)}
                          </span>

                          {/* Mini Custom Slider */}
                          <div className="w-24 h-1 bg-[#231E23] rounded-full relative">
                            <div 
                              className={`h-full rounded-full ${session.isMuted ? "bg-oto-slate/30" : "bg-oto-rose"}`}
                              style={{ width: `${session.isMuted ? 0 : session.volume}%` }}
                            ></div>
                            <div 
                              className={`w-2.5 h-2.5 rounded-full border border-oto-rose absolute top-1/2 -translate-y-1/2 -translate-x-1/2 transition-all ${
                                session.isMuted ? "bg-oto-slate" : "bg-oto-pink"
                              } ${isSelected ? "scale-125" : ""}`}
                              style={{ left: `${session.isMuted ? 0 : session.volume}%` }}
                            ></div>
                          </div>

                          {/* Accordion Indicator */}
                          {session.isDiscord && (
                            <span className="text-[10px] text-oto-slate w-3 text-center">
                              {isDiscordExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                            </span>
                          )}
                        </div>
                      </div>

                      {/* Discord Users Sub-List (Accordion Open) */}
                      {session.isDiscord && isDiscordExpanded && (
                        <div className="pl-6 pr-2 space-y-2 border-l border-oto-selection ml-3.5 my-2">
                          {discordUsers.map((user, uIndex) => {
                            const isUserSelected = uIndex === selectedDiscordIndex;
                            return (
                              <div
                                key={user.id}
                                data-selected={isUserSelected}
                                className={`flex items-center justify-between px-3 py-2 rounded transition-colors ${
                                  isUserSelected ? "bg-oto-selection/60 text-oto-white" : "bg-transparent text-oto-slate"
                                }`}
                              >
                                <div className="flex items-center gap-2">
                                  {isUserSelected && <div className="w-1 h-3 rounded bg-oto-rose"></div>}
                                  <span className="font-mono text-[11px]">{user.isMuted ? "🔇" : "🔊"} {user.username}</span>
                                </div>
                                
                                <div className="flex items-center gap-3">
                                  <span className="font-mono text-[11px] text-oto-rose">{user.isMuted ? "Muted" : `${user.volume}%`}</span>
                                  <div className="w-20 h-1 bg-[#231E23] rounded-full relative">
                                    <div 
                                      className={`h-full rounded-full ${user.isMuted ? "bg-oto-slate/30" : "bg-oto-rose"}`}
                                      style={{ width: `${user.isMuted ? 0 : user.volume / 2}%` }}
                                    ></div>
                                    <div 
                                      className={`w-2 h-2 rounded-full border border-oto-rose absolute top-1/2 -translate-y-1/2 -translate-x-1/2 ${
                                        user.isMuted ? "bg-oto-slate" : "bg-oto-pink"
                                      }`}
                                      style={{ left: `${user.isMuted ? 0 : user.volume / 2}%` }}
                                    ></div>
                                  </div>
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              {/* Input Shield overlay when not active */}
              {!isFocused && (
                <div className="absolute inset-0 bg-black/40 backdrop-blur-[1px] hover:bg-black/30 flex items-center justify-center transition-all duration-200">
                  <span className="px-5 py-2.5 bg-oto-panel border border-oto-selection text-oto-pink font-display text-sm rounded-lg hover:border-oto-pink hover:text-oto-white transition-all shadow-lg">
                    Click to Try
                  </span>
                </div>
              )}
            </div>

            {/* Instruction Badges */}
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-center text-[10px] font-mono text-oto-slate">
              <div className="bg-oto-panel/60 p-2 rounded border border-oto-selection/50">
                <span className="text-oto-pink font-bold">J</span> / <span className="text-oto-pink font-bold">K</span>
                <p className="mt-1 text-[9px]">Navigate Up/Down</p>
              </div>
              <div className="bg-oto-panel/60 p-2 rounded border border-oto-selection/50">
                <span className="text-oto-pink font-bold">H</span> / <span className="text-oto-pink font-bold">L</span>
                <p className="mt-1 text-[9px]">Volume Down/Up</p>
              </div>
              <div className="bg-oto-panel/60 p-2 rounded border border-oto-selection/50">
                <span className="text-oto-pink font-bold">M</span>
                <p className="mt-1 text-[9px]">Mute Selected</p>
              </div>
              <div className="bg-oto-panel/60 p-2 rounded border border-oto-selection/50">
                <span className="text-oto-pink font-bold">ENTER</span> / <span className="text-oto-pink font-bold">ESC</span>
                <p className="mt-1 text-[9px]">Open/Close Discord</p>
              </div>
            </div>

            {/* Mobile Interaction Pad */}
            {isMobile && isFocused && (
              <div className="bg-[#111114] p-4 rounded-xl border border-oto-selection flex flex-col gap-3">
                <div className="text-center text-[10px] font-mono text-oto-slate">Mobile Controls</div>
                <div className="flex justify-center gap-2">
                  <button 
                    onClick={() => {
                      if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
                        setSelectedDiscordIndex((prev) => Math.max(prev - 1, 0));
                      } else {
                        setSelectedIndex((prev) => Math.max(prev - 1, 0));
                      }
                      setLastKeyPressed("↓");
                    }}
                    className="flex-1 bg-oto-panel text-oto-white py-2 rounded text-xs font-mono border border-oto-selection active:bg-oto-selection"
                  >
                    Up
                  </button>
                  <button 
                    onClick={() => {
                      if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
                        setSelectedDiscordIndex((prev) => Math.min(prev + 1, discordUsers.length - 1));
                      } else {
                        setSelectedIndex((prev) => Math.min(prev + 1, sessions.length - 1));
                      }
                      setLastKeyPressed("↑");
                    }}
                    className="flex-1 bg-oto-panel text-oto-white py-2 rounded text-xs font-mono border border-oto-selection active:bg-oto-selection"
                  >
                    Down
                  </button>
                </div>
                <div className="flex gap-2">
                  <button 
                    onClick={() => {
                      if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
                        setDiscordUsers((users) => users.map((u, i) => i === selectedDiscordIndex ? { ...u, volume: Math.max(u.volume - 10, 0) } : u));
                      } else {
                        setSessions((prev) => prev.map((s, i) => i === selectedIndex ? { ...s, volume: Math.max(s.volume - 10, 0) } : s));
                      }
                      setLastKeyPressed("Vol -");
                    }}
                    className="flex-1 bg-oto-panel text-oto-rose py-2 rounded text-xs font-mono border border-oto-selection active:bg-oto-selection"
                  >
                    Vol -
                  </button>
                  <button 
                    onClick={() => {
                      if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
                        setDiscordUsers((users) => users.map((u, i) => i === selectedDiscordIndex ? { ...u, volume: Math.min(u.volume + 10, 200) } : u));
                      } else {
                        setSessions((prev) => prev.map((s, i) => i === selectedIndex ? { ...s, volume: Math.min(s.volume + 10, 100) } : s));
                      }
                      setLastKeyPressed("Vol +");
                    }}
                    className="flex-1 bg-oto-panel text-oto-rose py-2 rounded text-xs font-mono border border-oto-selection active:bg-oto-selection"
                  >
                    Vol +
                  </button>
                </div>
                <div className="flex gap-2">
                  {sessions[selectedIndex]?.isDiscord && (
                    <button 
                      onClick={() => {
                        setIsDiscordExpanded((prev) => !prev);
                        setSelectedDiscordIndex(0);
                      }}
                      className="flex-1 bg-oto-panel text-oto-white py-2 rounded text-xs font-mono border border-oto-selection active:bg-oto-selection"
                    >
                      {isDiscordExpanded ? "Collapse" : "Expand Discord"}
                    </button>
                  )}
                  <button 
                    onClick={() => {
                      if (isDiscordExpanded && sessions[selectedIndex]?.isDiscord) {
                        setDiscordUsers((users) => users.map((u, i) => i === selectedDiscordIndex ? { ...u, isMuted: !u.isMuted } : u));
                      } else {
                        setSessions((prev) => prev.map((s, i) => i === selectedIndex ? { ...s, isMuted: !s.isMuted } : s));
                      }
                    }}
                    className="flex-1 bg-oto-panel text-oto-rose py-2 rounded text-xs font-mono border border-oto-selection active:bg-oto-selection"
                  >
                    Mute
                  </button>
                </div>
              </div>
            )}
          </div>
        </section>

      </main>

      <section id="features" className="max-w-6xl mx-auto w-full px-6 py-16 md:py-24 border-t border-oto-selection/40">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-16">
          
          {/* Left Column: Core Features */}
          <div className="space-y-12">
            <div className="space-y-3">
              <h2 className="text-3xl font-display font-bold text-oto-pink">Keyboard-First Audio Control</h2>
              <p className="text-sm text-oto-slate leading-relaxed font-sans">
                Oto strips away mouse navigation entirely, allowing you to fine-tune your desktop audio pipeline instantly via a lightweight global keyboard overlay.
              </p>
            </div>

            <div className="space-y-8">
              {/* Feature 1 */}
              <div className="flex gap-4">
                <div className="w-10 h-10 shrink-0 rounded-lg bg-oto-pink/10 flex items-center justify-center text-oto-pink border border-oto-pink/30">
                  <Keyboard size={20} />
                </div>
                <div className="space-y-1">
                  <h3 className="font-display font-bold text-sm text-oto-white">Vim-like Bindings</h3>
                  <p className="text-xs text-oto-slate leading-relaxed font-sans">
                    Control active process volumes, mute apps, and select settings without reaching for your mouse. Built for developers and Vim power users.
                  </p>
                </div>
              </div>

              {/* Feature 2 */}
              <div className="flex gap-4">
                <div className="w-10 h-10 shrink-0 rounded-lg bg-oto-rose/10 flex items-center justify-center text-oto-rose border border-oto-rose/30">
                  <Zap size={20} />
                </div>
                <div className="space-y-1">
                  <h3 className="font-display font-bold text-sm text-oto-white">Zero Latency WASAPI</h3>
                  <p className="text-xs text-oto-slate leading-relaxed font-sans">
                    Direct integration with Windows WASAPI audio endpoints ensures lag-free volume routing, per-app mixing, and system-level audio responsiveness.
                  </p>
                </div>
              </div>

              {/* Feature 3 */}
              <div className="flex gap-4">
                <div className="w-10 h-10 shrink-0 rounded-lg bg-oto-pink/10 flex items-center justify-center text-oto-pink border border-oto-pink/30">
                  <Shield size={20} />
                </div>
                <div className="space-y-1">
                  <h3 className="font-display font-bold text-sm text-oto-white">Local Discord Control</h3>
                  <p className="text-xs text-oto-slate leading-relaxed font-sans">
                    Uses local Discord IPC socket communication to let you adjust per-user volume inside your voice channels directly, bypassing Discord's restrictive overlay whitelist.
                  </p>
                </div>
              </div>
            </div>
          </div>

          {/* Right Column: Keybinds Map */}
          <div className="p-8 rounded-2xl border border-oto-selection/60 space-y-6 flex flex-col justify-center">
            <h3 className="font-display font-bold text-base text-oto-white flex items-center gap-2 border-b border-oto-selection/40 pb-4">
              <Keyboard size={18} className="text-oto-pink" />
              <span>Default Keybindings Reference</span>
            </h3>
            
            <div className="space-y-4 font-mono text-[11px]">
              <div className="flex items-center justify-between border-b border-oto-selection/20 pb-3">
                <span className="text-oto-slate">Toggle Overlay</span>
                <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">`</span>
              </div>

              <div className="flex items-center justify-between border-b border-oto-selection/20 pb-3">
                <span className="text-oto-slate">Select app (Down / Up)</span>
                <div className="flex gap-1">
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">J</span>
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">K</span>
                </div>
              </div>

              <div className="flex items-center justify-between border-b border-oto-selection/20 pb-3">
                <span className="text-oto-slate">Change volume (- / +)</span>
                <div className="flex gap-1">
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">H</span>
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">L</span>
                </div>
              </div>

              <div className="flex items-center justify-between border-b border-oto-selection/20 pb-3">
                <span className="text-oto-slate">Fast volume adjust (step: 10)</span>
                <div className="flex gap-1.5 items-center">
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">Shift</span>
                  <span>+</span>
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">H</span>
                  <span>/</span>
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">L</span>
                </div>
              </div>

              <div className="flex items-center justify-between border-b border-oto-selection/20 pb-3">
                <span className="text-oto-slate">Toggle Mute</span>
                <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">M</span>
              </div>

              <div className="flex items-center justify-between border-b border-oto-selection/20 pb-3">
                <span className="text-oto-slate">Discord User List (Expand / Collapse)</span>
                <div className="flex gap-1">
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">Enter</span>
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">Esc</span>
                </div>
              </div>

              <div className="flex items-center justify-between border-b border-oto-selection/20 pb-3">
                <span className="text-oto-slate">Jump to Top / Bottom</span>
                <div className="flex gap-1">
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">GG</span>
                  <span>/</span>
                  <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">G</span>
                </div>
              </div>

              <div className="flex items-center justify-between border-b border-oto-selection/20 pb-3">
                <span className="text-oto-slate">Toggle PTT Mode</span>
                <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">T</span>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-oto-slate">Push-to-Talk (Hold)</span>
                <span className="px-2 py-0.5 bg-oto-selection text-oto-pink rounded border border-oto-pink/20">V</span>
              </div>
            </div>
          </div>

        </div>
      </section>

      {/* DISCORD SETUP GUIDE */}
      <section id="docs" className="max-w-6xl mx-auto w-full px-6 py-16 md:py-24 border-t border-oto-selection/40">
        {/* Section Header */}
        <div className="space-y-3 mb-12">
          <h2 className="text-3xl font-display font-bold text-oto-pink">Discord API Setup</h2>
          <p className="text-sm text-oto-slate max-w-2xl leading-relaxed font-sans font-light">
            Because Discord restricts local RPC socket access to whitelisted games, you must configure a private developer application. This runs entirely locally on your machine, ensuring 100% decentralized data privacy.
          </p>
        </div>

        {/* Content Split */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-start">
          {/* Left: Redirect Info Panel */}
          <div className="lg:col-span-5 p-6 rounded-xl border border-oto-selection/60 space-y-4">
            <h3 className="font-display font-bold text-sm text-oto-white flex items-center gap-2">
              <Info size={16} className="text-oto-pink shrink-0" />
              <span>OAuth2 Redirect Configuration</span>
            </h3>
            <p className="text-xs text-oto-slate leading-relaxed font-sans font-light">
              During developer setup, Discord requires a designated redirect URL. Use the local address below:
            </p>
            <div className="p-3.5 bg-[#101012] rounded-lg border border-oto-selection font-mono text-[11px] text-oto-pink flex items-center justify-between">
              <code>http://127.0.0.1</code>
              <span className="text-[9px] text-oto-slate uppercase tracking-wider">Local Only</span>
            </div>
          </div>

          {/* Right: Step List */}
          <div className="lg:col-span-7 space-y-6">
            <div className="flex gap-4">
              <div className="w-6 h-6 rounded-full bg-oto-pink/10 border border-oto-pink/30 flex items-center justify-center font-mono text-[10px] text-oto-pink shrink-0 mt-0.5 font-bold">1</div>
              <div className="space-y-1">
                <h4 className="text-xs font-mono font-semibold text-oto-white">Create Developer Application</h4>
                <p className="text-xs text-oto-slate leading-relaxed font-sans font-light">
                  Go to the <a href="https://discord.com/developers/applications" target="_blank" rel="noopener noreferrer" className="text-oto-pink hover:underline font-semibold">Discord Developer Portal</a> and click <strong>New Application</strong>. Choose a name like "Oto Mixer".
                </p>
              </div>
            </div>

            <div className="flex gap-4">
              <div className="w-6 h-6 rounded-full bg-oto-rose/10 border border-oto-rose/30 flex items-center justify-center font-mono text-[10px] text-oto-rose shrink-0 mt-0.5 font-bold">2</div>
              <div className="space-y-1">
                <h4 className="text-xs font-mono font-semibold text-oto-white">Configure OAuth2 Settings</h4>
                <p className="text-xs text-oto-slate leading-relaxed font-sans font-light">
                  Select <strong>OAuth2</strong> from the left menu. Under <strong>Redirects</strong>, add <code>http://127.0.0.1</code> and save changes.
                </p>
              </div>
            </div>

            <div className="flex gap-4">
              <div className="w-6 h-6 rounded-full bg-oto-pink/10 border border-oto-pink/30 flex items-center justify-center font-mono text-[10px] text-oto-pink shrink-0 mt-0.5 font-bold">3</div>
              <div className="space-y-1">
                <h4 className="text-xs font-mono font-semibold text-oto-white">Authenticate Local Client</h4>
                <p className="text-xs text-oto-slate leading-relaxed font-sans font-light">
                  Copy your <strong>Client ID</strong> and <strong>Client Secret</strong> (reset secret if needed). Open Oto's settings (⚙ Gear Icon), paste them in, and click save. On restart, authorize the popup window to grant local RPC access.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* FAQ SECTION */}
      <section className="max-w-6xl mx-auto w-full px-6 py-16 md:py-24 border-t border-oto-selection/40">
        {/* Section Header */}
        <div className="space-y-3 mb-16">
          <h2 className="text-3xl font-display font-bold text-oto-pink">Frequently Asked Questions</h2>
          <p className="text-sm text-oto-slate max-w-2xl leading-relaxed font-sans font-light">
            Everything you need to know about setting up and running Oto on your Windows system.
          </p>
        </div>

        {/* FAQ Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-x-12 gap-y-10 font-sans text-xs">
          <div className="space-y-2 border-l-2 border-oto-pink/30 pl-4 py-1">
            <h4 className="font-mono font-semibold text-oto-white text-xs">Does Oto require Administrator privileges?</h4>
            <p className="text-oto-slate leading-relaxed font-light">
              No. Oto runs entirely in standard user-space. It uses native Windows WASAPI COM endpoints to adjust session volumes without needing elevated system administrative rights.
            </p>
          </div>
          <div className="space-y-2 border-l-2 border-oto-pink/30 pl-4 py-1">
            <h4 className="font-mono font-semibold text-oto-white text-xs">Why does Discord require a custom Client ID?</h4>
            <p className="text-oto-slate leading-relaxed font-light">
              Discord limits local RPC socket integrations. Using private developer credentials bypasses these restrictions, keeping Oto serverless and private.
            </p>
          </div>
          <div className="space-y-2 border-l-2 border-oto-pink/30 pl-4 py-1">
            <h4 className="font-mono font-semibold text-oto-white text-xs">Is the overlay visible in full-screen games?</h4>
            <p className="text-oto-slate leading-relaxed font-light">
              Yes. For optimal overlay drawing and zero performance impact, we recommend running your games in **Borderless Windowed** mode.
            </p>
          </div>
          <div className="space-y-2 border-l-2 border-oto-pink/30 pl-4 py-1">
            <h4 className="font-mono font-semibold text-oto-white text-xs">What is the performance footprint?</h4>
            <p className="text-oto-slate leading-relaxed font-light">
              Oto is written in native Rust. It operates in the background using **less than 10MB of RAM** and practically **0% CPU**, ensuring zero impact on your framerates.
            </p>
          </div>
        </div>
      </section>

      {/* FINAL CALL TO ACTION */}
      <section className="max-w-6xl mx-auto w-full px-6 pb-24 text-center">
        <div className="p-12 space-y-8">
          <div className="space-y-3">
            <h2 className="text-3xl md:text-4xl font-display font-bold text-oto-white">Take control of your audio workflow.</h2>
            <p className="text-xs md:text-sm text-oto-slate max-w-lg mx-auto font-sans font-light">
              Download the Windows installer and start mixing your audio instantly without touching your mouse. 100% free and open-source.
            </p>
          </div>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <a
              href="https://github.com/Starz099/oto/releases/download/v0.1.2/oto-0.1.2-x86_64.msi"
              className="relative overflow-hidden w-full sm:w-auto px-6 py-3.5 bg-oto-pink text-oto-dark font-sans font-semibold rounded-lg hover:bg-oto-rose hover:text-oto-dark transition-all duration-200 text-center shadow-lg shadow-oto-pink/10 flex items-center justify-center gap-2 group"
            >
              <Download size={16} className="group-hover:translate-y-0.5 transition-transform relative z-10" />
              <span className="relative z-10">Download Now</span>
              <span className="custom-shine-element" />
            </a>
            <a
              href="https://github.com/Starz099/oto"
              target="_blank"
              rel="noopener noreferrer"
              className="relative overflow-hidden w-full sm:w-auto px-6 py-3.5 bg-[#141414] hover:bg-oto-selection text-oto-white font-sans font-semibold rounded-lg border border-oto-selection transition-all text-center flex items-center justify-center gap-2"
            >
              <Github size={16} className="relative z-10" />
              <span className="relative z-10">GitHub Repository</span>
              <span className="custom-shine-element" />
            </a>
          </div>
        </div>
      </section>

      {/* FOOTER */}
      <footer className="max-w-6xl mx-auto w-full px-6 py-8 border-t border-oto-selection/40 flex flex-col md:flex-row items-center justify-between gap-4 font-mono text-[11px] text-oto-slate">
        <div>
          <span>
            made with love by{" "}
            <a 
              href="https://starzz.dev" 
              target="_blank" 
              rel="noopener noreferrer" 
              className="text-oto-rose hover:text-oto-pink transition-colors font-bold"
            >
              Starz099
            </a>
          </span>
        </div>
        <div>
          <a 
            href="https://github.com/Starz099/oto" 
            target="_blank" 
            rel="noopener noreferrer" 
            className="text-oto-rose hover:text-oto-pink font-bold transition-colors flex items-center gap-1.5"
          >
            <Github size={12} />
            <span>Repository</span>
          </a>
        </div>
      </footer>

    </div>
  );
}
