"use client";

import { useState, useEffect } from "react";
import { Search, X } from "lucide-react";

interface GlobalSearchProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function GlobalSearch({ isOpen, onClose }: GlobalSearchProps) {
  const [query, setQuery] = useState("");

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener("keydown", handleKeyDown);
      document.body.style.overflow = "hidden";
    }

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.body.style.overflow = "unset";
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 z-[100] flex items-start justify-center pt-20">
      <div className="bg-white dark:bg-slate-900 rounded-lg shadow-xl w-full max-w-2xl mx-4 border border-slate-200 dark:border-slate-700">
        <div className="flex items-center gap-3 p-4 border-b border-slate-200 dark:border-slate-700">
          <Search size={20} className="text-slate-400" />
          <input
            type="text"
            placeholder="Search..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="flex-1 bg-transparent text-slate-900 dark:text-white placeholder-slate-400 outline-none"
            autoFocus
          />
          <button
            onClick={onClose}
            className="p-1 hover:bg-slate-100 dark:hover:bg-slate-800 rounded"
          >
            <X size={16} className="text-slate-400" />
          </button>
        </div>
        
        <div className="p-4 max-h-96 overflow-y-auto">
          {query ? (
            <div className="text-slate-500 dark:text-slate-400 text-sm">
              Search results for {'"'}{query}{'"'} would appear here
            </div>
          ) : (
            <div className="text-slate-500 dark:text-slate-400 text-sm">
              Start typing to search...
            </div>
          )}
        </div>
      </div>
    </div>
  );
}