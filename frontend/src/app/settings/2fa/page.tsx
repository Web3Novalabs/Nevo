"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/Button";
import { QRCodeSVG } from "qrcode.react";

type TwoFactorMethod = "totp" | "sms" | null;

interface BackupCode {
  code: string;
  used: boolean;
}

export default function TwoFactorSetupPage() {
  const [is2FAEnabled, setIs2FAEnabled] = useState(false);
  const [method, setMethod] = useState<TwoFactorMethod>(null);
  const [step, setStep] = useState<"select" | "setup" | "verify" | "backup" | "success">("select");
  const [secret, setSecret] = useState("");
  const [qrCodeUrl, setQrCodeUrl] = useState("");
  const [phoneNumber, setPhoneNumber] = useState("");
  const [verificationCode, setVerificationCode] = useState("");
  const [backupCodes, setBackupCodes] = useState<BackupCode[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [copied, setCopied] = useState(false);

  // Fetch 2FA status on mount
  useEffect(() => {
    fetch2FAStatus();
  }, []);

  const fetch2FAStatus = async () => {
    try {
      const response = await fetch("/api/auth/2fa/status");
      const data = await response.json();
      setIs2FAEnabled(data.enabled);
      setMethod(data.method);
    } catch (err) {
      console.error("Failed to fetch 2FA status");
    }
  };

  const handleMethodSelect = async (selectedMethod: "totp" | "sms") => {
    setMethod(selectedMethod);
    setStep("setup");
    setIsLoading(true);
    setError("");

    try {
      const response = await fetch("/api/auth/2fa/setup", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ method: selectedMethod }),
      });

      const data = await response.json();

      if (response.ok) {
        if (selectedMethod === "totp") {
          setSecret(data.secret);
          setQrCodeUrl(data.qrCodeUrl);
        } else {
          // SMS - code sent to phone
          setPhoneNumber(data.maskedPhone);
        }
      } else {
        setError(data.error || "Failed to setup 2FA");
        setStep("select");
      }
    } catch (err) {
      setError("An error occurred. Please try again.");
      setStep("select");
    } finally {
      setIsLoading(false);
    }
  };

  const handleVerifyCode = async () => {
    if (!verificationCode) {
      setError("Please enter the verification code");
      return;
    }

    setIsLoading(true);
    setError("");

    try {
      const response = await fetch("/api/auth/2fa/verify-setup", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ 
          code: verificationCode,
          method,
          ...(method === "sms" && { phone: phoneNumber })
        }),
      });

      const data = await response.json();

      if (response.ok) {
        setBackupCodes(data.backupCodes);
        setStep("backup");
      } else {
        setError(data.error || "Invalid verification code");
      }
    } catch (err) {
      setError("An error occurred. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  const handleBackupCodesConfirmation = () => {
    setStep("success");
    setIs2FAEnabled(true);
  };

  const handleDisable2FA = async () => {
    if (!confirm("Are you sure you want to disable 2FA? This will make your account less secure.")) {
      return;
    }

    setIsLoading(true);
    setError("");

    try {
      const response = await fetch("/api/auth/2fa/disable", {
        method: "POST",
      });

      if (response.ok) {
        setIs2FAEnabled(false);
        setMethod(null);
        setStep("select");
        setSecret("");
        setQrCodeUrl("");
        setBackupCodes([]);
      } else {
        const data = await response.json();
        setError(data.error || "Failed to disable 2FA");
      }
    } catch (err) {
      setError("An error occurred. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  const copyBackupCodes = () => {
    const codes = backupCodes.map(c => c.code).join("\n");
    navigator.clipboard.writeText(codes);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const downloadBackupCodes = () => {
    const codes = backupCodes.map(c => c.code).join("\n");
    const blob = new Blob([`Your Backup Codes:\n\n${codes}\n\nKeep these codes safe! Each code can only be used once.`], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "nevobo-backup-codes.txt";
    a.click();
    URL.revokeObjectURL(url);
  };

  // 2FA Already Enabled
  if (is2FAEnabled && step === "select") {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 px-4 py-12">
        <div className="max-w-2xl mx-auto">
          <div className="bg-white rounded-2xl shadow-xl p-8">
            <div className="text-center mb-8">
              <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <span className="text-3xl">🔒</span>
              </div>
              <h1 className="text-3xl font-bold text-slate-900 mb-2">Two-Factor Authentication</h1>
              <p className="text-slate-600">Your account is protected with 2FA</p>
            </div>

            <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-6">
              <div className="flex items-center gap-3">
                <span className="text-green-600 text-xl">✓</span>
                <div>
                  <p className="font-medium text-green-800">2FA is enabled</p>
                  <p className="text-sm text-green-600">
                    Method: {method === "totp" ? "Authenticator App" : "SMS"}
                  </p>
                </div>
              </div>
            </div>

            <div className="space-y-4">
              <button
                onClick={() => setStep("setup")}
                className="w-full p-4 border border-slate-200 rounded-lg hover:border-indigo-500 hover:bg-indigo-50 transition-colors text-left"
              >
                <h3 className="font-medium text-slate-900">Regenerate Backup Codes</h3>
                <p className="text-sm text-slate-500">Get new backup codes for account recovery</p>
              </button>

              <button
                onClick={handleDisable2FA}
                className="w-full p-4 border border-red-200 rounded-lg hover:bg-red-50 transition-colors text-left"
              >
                <h3 className="font-medium text-red-600">Disable 2FA</h3>
                <p className="text-sm text-red-500">Remove two-factor authentication from your account</p>
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 px-4 py-12">
      <div className="max-w-2xl mx-auto">
        <div className="bg-white rounded-2xl shadow-xl p-8">
          {/* Step 1: Select Method */}
          {step === "select" && (
            <>
              <div className="text-center mb-8">
                <h1 className="text-3xl font-bold text-slate-900 mb-2">Enable Two-Factor Authentication</h1>
                <p className="text-slate-600">Add an extra layer of security to your account</p>
              </div>

              {error && (
                <div className="mb-6 p-4 bg-red-50 text-red-800 rounded-lg border border-red-200">
                  {error}
                </div>
              )}

              <div className="space-y-4">
                <button
                  onClick={() => handleMethodSelect("totp")}
                  disabled={isLoading}
                  className="w-full p-6 border border-slate-200 rounded-lg hover:border-indigo-500 hover:bg-indigo-50 transition-colors text-left disabled:opacity-50"
                >
                  <div className="flex items-start gap-4">
                    <div className="w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center flex-shrink-0">
                      <span className="text-2xl">📱</span>
                    </div>
                    <div>
                      <h3 className="font-semibold text-slate-900 mb-1">Authenticator App</h3>
                      <p className="text-sm text-slate-500">
                        Use an app like Google Authenticator, Authy, or 1Password to generate codes
                      </p>
                    </div>
                  </div>
                </button>

                <button
                  onClick={() => handleMethodSelect("sms")}
                  disabled={isLoading}
                  className="w-full p-6 border border-slate-200 rounded-lg hover:border-indigo-500 hover:bg-indigo-50 transition-colors text-left disabled:opacity-50"
                >
                  <div className="flex items-start gap-4">
                    <div className="w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center flex-shrink-0">
                      <span className="text-2xl">💬</span>
                    </div>
                    <div>
                      <h3 className="font-semibold text-slate-900 mb-1">SMS Verification</h3>
                      <p className="text-sm text-slate-500">
                        Receive verification codes via text message
                      </p>
                    </div>
                  </div>
                </button>
              </div>
            </>
          )}

          {/* Step 2: Setup (QR Code or SMS) */}
          {step === "setup" && (
            <>
              <div className="text-center mb-8">
                <h1 className="text-3xl font-bold text-slate-900 mb-2">
                  {method === "totp" ? "Set up Authenticator" : "Verify Phone Number"}
                </h1>
              </div>

              {error && (
                <div className="mb-6 p-4 bg-red-50 text-red-800 rounded-lg border border-red-200">
                  {error}
                </div>
              )}

              {method === "totp" ? (
                <div className="text-center">
                  <div className="inline-block p-4 bg-white border border-slate-200 rounded-lg mb-6">
                    <QRCodeSVG value={qrCodeUrl} size={200} level="H" />
                  </div>
                  
                  <div className="mb-6">
                    <p className="text-sm text-slate-600 mb-2">Scan this QR code with your authenticator app</p>
                    <p className="text-xs text-slate-500 font-mono bg-slate-100 p-2 rounded">
                      {secret}
                    </p>
                  </div>

                  <div className="space-y-4">
                    <input
                      type="text"
                      placeholder="Enter 6-digit code"
                      value={verificationCode}
                      onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, "").slice(0, 6))}
                      className="w-full px-4 py-3 text-center text-2xl tracking-widest rounded-lg border border-slate-300 focus:ring-2 focus:ring-indigo-500"
                      maxLength={6}
                    />
                    <Button onClick={handleVerifyCode} className="w-full" disabled={isLoading}>
                      {isLoading ? "Verifying..." : "Verify & Enable 2FA"}
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-2">
                      Phone Number
                    </label>
                    <input
                      type="tel"
                      placeholder="+1 (555) 123-4567"
                      value={phoneNumber}
                      onChange={(e) => setPhoneNumber(e.target.value)}
                      className="w-full px-4 py-3 rounded-lg border border-slate-300 focus:ring-2 focus:ring-indigo-500"
                    />
                  </div>

                  <div className="flex gap-4">
                    <Button variant="outline" onClick={() => setStep("select")} className="flex-1">
                      Back
                    </Button>
                    <Button onClick={handleVerifyCode} className="flex-1" disabled={isLoading}>
                      {isLoading ? "Sending..." : "Send Code"}
                    </Button>
                  </div>

                  {verificationCode && (
                    <div className="space-y-4 pt-4 border-t">
                      <p className="text-sm text-slate-600">Enter the code sent to your phone:</p>
                      <input
                        type="text"
                        placeholder="Enter 6-digit code"
                        value={verificationCode}
                        onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, "").slice(0, 6))}
                        className="w-full px-4 py-3 text-center text-2xl tracking-widest rounded-lg border border-slate-300 focus:ring-2 focus:ring-indigo-500"
                        maxLength={6}
                      />
                      <Button onClick={handleVerifyCode} className="w-full" disabled={isLoading}>
                        {isLoading ? "Verifying..." : "Verify & Enable 2FA"}
                      </Button>
                    </div>
                  )}
                </div>
              )}
            </>
          )}

          {/* Step 3: Backup Codes */}
          {step === "backup" && (
            <>
              <div className="text-center mb-8">
                <div className="w-16 h-16 bg-yellow-100 rounded-full flex items-center justify-center mx-auto mb-4">
                  <span className="text-3xl">⚠️</span>
                </div>
                <h1 className="text-3xl font-bold text-slate-900 mb-2">Save Your Backup Codes</h1>
                <p className="text-slate-600">
                  Store these codes safely. Each code can only be used once.
                </p>
              </div>

              <div className="bg-slate-900 rounded-lg p-6 mb-6">
                <div className="grid grid-cols-2 gap-2">
                  {backupCodes.map((item, index) => (
                    <code key={index} className="text-green-400 font-mono text-sm">
                      {item.code}
                    </code>
                  ))}
                </div>
              </div>

              <div className="flex gap-4 mb-6">
                <Button variant="outline" onClick={copyBackupCodes} className="flex-1">
                  {copied ? "Copied!" : "Copy Codes"}
                </Button>
                <Button variant="outline" onClick={downloadBackupCodes} className="flex-1">
                  Download
                </Button>
              </div>

              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
                <p className="text-sm text-yellow-800">
                  <strong>Important:</strong> Store these codes in a safe place. You'll need them if you lose access to your authenticator app or phone.
                </p>
              </div>

              <Button onClick={handleBackupCodesConfirmation} className="w-full">
                I've Saved My Codes - Complete Setup
              </Button>
            </>
          )}

          {/* Step 4: Success */}
          {step === "success" && (
            <div className="text-center">
              <div className="w-20 h-20 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-6">
                <span className="text-4xl">✓</span>
              </div>
              <h1 className="text-3xl font-bold text-slate-900 mb-2">2FA Enabled Successfully!</h1>
              <p className="text-slate-600 mb-8">
                Your account is now protected with two-factor authentication
              </p>
              <Button onClick={() => window.location.href = "/dashboard"}>
                Continue to Dashboard
              </Button>
            </div>
          )}

          {/* Loading State */}
          {isLoading && step === "setup" && (
            <div className="text-center py-8">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
              <p className="mt-4 text-slate-600">Setting up 2FA...</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}