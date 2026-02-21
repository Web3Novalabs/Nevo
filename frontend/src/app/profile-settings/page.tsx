"use client";

import React, { useState, useEffect } from "react";
import Navigation from "@/components/Navigation";
import Footer from "@/components/Footer";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Camera, UserCircle } from "lucide-react";
import { toast } from "sonner";

// Dummy initial profile
const defaultProfile = {
  displayName: "Alex Nakamoto",
  avatarUrl: "",
};

export default function ProfileSettingsPage() {
  const [isClient, setIsClient] = useState(false);
  const [profile, setProfile] = useState<{ displayName: string; avatarUrl: string }>(defaultProfile);
  const [formData, setFormData] = useState({ displayName: "", avatarUrl: "" });
  const [isEditing, setIsEditing] = useState(false);

  useEffect(() => {
    setIsClient(true);
    // On component mount, attempt to retrieve from localStorage
    const storedStr = localStorage.getItem("nevo_user_profile");
    if (storedStr) {
      try {
        const stored = JSON.parse(storedStr);
        setProfile(stored);
        setFormData(stored);
      } catch {
        setFormData(defaultProfile);
      }
    } else {
      setFormData(defaultProfile);
    }
  }, []);

  const handleSave = () => {
    // Save to LocalStorage
    localStorage.setItem("nevo_user_profile", JSON.stringify(formData));
    setProfile(formData);
    setIsEditing(false);
    toast.success("Profile saved successfully!");
  };

  const handleCancel = () => {
    setFormData(profile);
    setIsEditing(false);
  };

  if (!isClient) return null; // Avoid hydration mismatch

  return (
    <div className="bg-[#0F172A] min-h-screen flex flex-col font-dmsans">
      <Navigation />

      <main className="flex-grow pt-24 pb-20 px-4 sm:px-6 lg:px-8 mt-14">
        <div className="max-w-3xl mx-auto">
          {/* Header */}
          <div className="mb-10 text-center md:text-left">
            <h1 className="text-3xl md:text-4xl font-extrabold text-white tracking-tight">
              Profile Settings
            </h1>
            <p className="text-slate-400 mt-2">
              Manage your connected account identity.
            </p>
          </div>

          <div className="bg-[#1E293B] border border-slate-700/50 rounded-2xl p-6 md:p-8 shadow-2xl">
            <div className="flex flex-col md:flex-row gap-10 items-start">
              
              {/* Avatar Section */}
              <div className="flex flex-col items-center gap-4 w-full md:w-auto">
                <div className="relative group">
                  <div className="w-32 h-32 rounded-full overflow-hidden bg-slate-800 border-4 border-slate-700 flex items-center justify-center relative shadow-inner">
                    {formData.avatarUrl ? (
                      /* eslint-disable-next-line @next/next/no-img-element */
                      <img
                        src={formData.avatarUrl}
                        alt="Avatar"
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <UserCircle className="w-20 h-20 text-slate-500" />
                    )}
                    {isEditing && (
                      <div className="absolute inset-0 bg-black/50 flex flex-col items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer backdrop-blur-sm">
                        <Camera className="w-6 h-6 text-white mb-1" />
                        <span className="text-xs text-white font-medium">Edit URL</span>
                      </div>
                    )}
                  </div>
                </div>
                {isEditing && (
                  <div className="w-full">
                    <Label htmlFor="avatarUrl" className="text-slate-400 text-xs mb-1.5 block text-center font-medium">Avatar Image URL</Label>
                    <Input
                      id="avatarUrl"
                      type="text"
                      placeholder="https://example.com/avatar.jpg"
                      value={formData.avatarUrl}
                      onChange={(e) => setFormData({ ...formData, avatarUrl: e.target.value })}
                      className="bg-slate-900 border-slate-700 text-white text-sm"
                    />
                  </div>
                )}
              </div>

              {/* Details Section */}
              <div className="flex-grow w-full space-y-6">
                <div>
                  <Label htmlFor="displayName" className="text-slate-300 mb-2 block font-medium">Display Name</Label>
                  <Input
                    id="displayName"
                    type="text"
                    value={formData.displayName}
                    onChange={(e) => setFormData({ ...formData, displayName: e.target.value })}
                    disabled={!isEditing}
                    className="bg-slate-900 border-slate-700 text-white text-lg h-12 disabled:opacity-60 disabled:cursor-not-allowed focus-visible:ring-blue-500"
                  />
                  {!isEditing && (
                    <p className="text-sm text-slate-500 mt-2">
                      This is how you will appear to other users in Nevo.
                    </p>
                  )}
                </div>

                <div className="pt-4 flex gap-3">
                  {isEditing ? (
                    <>
                      <Button onClick={handleSave} className="bg-blue-600 hover:bg-blue-700 text-white">
                        Save Changes
                      </Button>
                      <Button variant="outline" onClick={handleCancel} className="bg-transparent border-slate-600 text-slate-300 hover:bg-slate-800 hover:text-white">
                        Cancel
                      </Button>
                    </>
                  ) : (
                    <Button onClick={() => setIsEditing(true)} className="bg-slate-700 hover:bg-slate-600 text-white">
                      Edit Profile
                    </Button>
                  )}
                </div>
              </div>

            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
