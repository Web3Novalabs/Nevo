"use client";

import { useState, useEffect } from "react";
import {
  ArrowLeft,
  Loader2,
  CheckCircle2,
  XCircle,
  ExternalLink,
  Calendar,
  DollarSign,
  FileText,
  Image as ImageIcon,
  Link as LinkIcon,
} from "lucide-react";
import Navigation from "@/components/Navigation";
import Footer from "@/components/Footer";
import Link from "next/link";
import { getPublicKey, connect } from "@/app/stellar-wallets-kit";
import { createPool, getStellarExpertUrl } from "@/utils/soroban";

interface FormData {
  name: string;
  description: string;
  externalUrl: string;
  imageHash: string;
  targetAmount: string;
  durationDays: string;
}

type SubmissionState = "idle" | "submitting" | "success" | "error";

export default function CreatePoolPage() {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [isCheckingWallet, setIsCheckingWallet] = useState(true);
  const [formData, setFormData] = useState<FormData>({
    name: "",
    description: "",
    externalUrl: "",
    imageHash: "",
    targetAmount: "",
    durationDays: "",
  });
  const [submissionState, setSubmissionState] = useState<SubmissionState>("idle");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [txHash, setTxHash] = useState<string>("");
  const [poolId, setPoolId] = useState<number | null>(null);

  // Check wallet connection on mount
  useEffect(() => {
    const checkWallet = async () => {
      const key = await getPublicKey();
      setPublicKey(key);
      setIsCheckingWallet(false);
    };
    checkWallet();
  }, []);

  const handleConnectWallet = async () => {
    await connect(async () => {
      const key = await getPublicKey();
      setPublicKey(key);
    });
  };

  const handleInputChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  const validateForm = (): boolean => {
    if (!formData.name.trim()) {
      setErrorMessage("Pool name is required");
      return false;
    }
    if (!formData.description.trim()) {
      setErrorMessage("Description is required");
      return false;
    }
    if (formData.description.length > 500) {
      setErrorMessage("Description must be 500 characters or less");
      return false;
    }
    if (!formData.externalUrl.trim()) {
      setErrorMessage("External URL is required");
      return false;
    }
    if (formData.externalUrl.length > 200) {
      setErrorMessage("External URL must be 200 characters or less");
      return false;
    }
    if (!formData.imageHash.trim()) {
      setErrorMessage("Image hash is required");
      return false;
    }
    if (formData.imageHash.length > 100) {
      setErrorMessage("Image hash must be 100 characters or less");
      return false;
    }
    if (!formData.targetAmount || parseFloat(formData.targetAmount) <= 0) {
      setErrorMessage("Target amount must be greater than 0");
      return false;
    }
    if (!formData.durationDays || parseInt(formData.durationDays) <= 0) {
      setErrorMessage("Duration must be greater than 0 days");
      return false;
    }
    return true;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!publicKey) {
      setErrorMessage("Please connect your wallet first");
      return;
    }

    if (!validateForm()) {
      return;
    }

    setSubmissionState("submitting");
    setErrorMessage("");

    try {
      // Calculate deadline: current time + duration in seconds
      const now = Math.floor(Date.now() / 1000);
      const durationSeconds = parseInt(formData.durationDays) * 24 * 60 * 60;
      const deadline = now + durationSeconds;

      const result = await createPool({
        name: formData.name,
        description: formData.description,
        externalUrl: formData.externalUrl,
        imageHash: formData.imageHash,
        targetAmount: formData.targetAmount,
        deadline,
      });

      setTxHash(result.txHash);
      setPoolId(result.poolId);
      setSubmissionState("success");
    } catch (error: any) {
      console.error("Pool creation error:", error);
      setSubmissionState("error");
      
      // Handle specific error types
      if (error.message?.includes("User rejected")) {
        setErrorMessage("Transaction was rejected. Please try again.");
      } else if (error.message?.includes("Simulation failed")) {
        setErrorMessage("Transaction simulation failed. Please check your inputs.");
      } else if (error.message?.includes("Transaction failed")) {
        setErrorMessage(`Transaction failed: ${error.message}`);
      } else {
        setErrorMessage(
          error.message || "Failed to create pool. Please try again."
        );
      }
    }
  };

  if (isCheckingWallet) {
    return (
      <div className="bg-[#0F172A] min-h-screen flex items-center justify-center">
        <Loader2 className="animate-spin text-[#50C878]" size={32} />
      </div>
    );
  }

  return (
    <div className="bg-[#0F172A] min-h-screen">
      <Navigation />

      <div className="pt-32 pb-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-3xl mx-auto">
          {/* Back Button */}
          <Link
            href="/discovery"
            className="inline-flex items-center gap-2 text-slate-400 hover:text-[#50C878] transition-colors mb-8"
          >
            <ArrowLeft size={20} />
            <span>Back to Discovery</span>
          </Link>

          {/* Header */}
          <div className="mb-10">
            <h1 className="text-4xl font-bold text-white mb-4">
              Create a Donation Pool
            </h1>
            <p className="text-slate-400 text-lg">
              Launch a transparent donation pool on the Stellar blockchain.
              Fill out the form below to get started.
            </p>
          </div>

          {/* Wallet Connection Check */}
          {!publicKey && (
            <div className="bg-slate-800/60 border border-slate-700/50 rounded-2xl p-6 mb-8">
              <p className="text-slate-300 mb-4">
                Please connect your wallet to create a pool.
              </p>
              <button
                onClick={handleConnectWallet}
                className="bg-[#50C878] text-black px-6 py-3 rounded-lg font-semibold hover:brightness-110 transition"
              >
                Connect Wallet
              </button>
            </div>
          )}

          {/* Success State */}
          {submissionState === "success" && (
            <div className="bg-gradient-to-br from-emerald-500/10 to-green-500/10 border border-emerald-500/30 rounded-2xl p-8 mb-8">
              <div className="flex items-start gap-4">
                <CheckCircle2 className="text-emerald-400 flex-shrink-0 mt-1" size={32} />
                <div className="flex-1">
                  <h2 className="text-2xl font-bold text-white mb-2">
                    Pool Created Successfully!
                  </h2>
                  <p className="text-slate-300 mb-4">
                    Your donation pool has been created on the Stellar blockchain.
                    {poolId && (
                      <span className="block mt-2">
                        Pool ID: <span className="font-mono text-[#50C878]">{poolId}</span>
                      </span>
                    )}
                  </p>
                  {txHash && (
                    <a
                      href={getStellarExpertUrl(txHash)}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-2 text-[#50C878] hover:text-emerald-300 transition-colors font-medium"
                    >
                      View on Stellar Expert
                      <ExternalLink size={16} />
                    </a>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Error State */}
          {submissionState === "error" && (
            <div className="bg-gradient-to-br from-red-500/10 to-rose-500/10 border border-red-500/30 rounded-2xl p-8 mb-8">
              <div className="flex items-start gap-4">
                <XCircle className="text-red-400 flex-shrink-0 mt-1" size={32} />
                <div className="flex-1">
                  <h2 className="text-2xl font-bold text-white mb-2">
                    Transaction Failed
                  </h2>
                  <p className="text-slate-300 mb-4">{errorMessage}</p>
                  <button
                    onClick={() => {
                      setSubmissionState("idle");
                      setErrorMessage("");
                    }}
                    className="bg-slate-700 hover:bg-slate-600 text-white px-6 py-2 rounded-lg font-semibold transition"
                  >
                    Try Again
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Form */}
          {submissionState !== "success" && publicKey && (
            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Pool Name */}
              <div className="bg-slate-800/60 border border-slate-700/50 rounded-2xl p-6">
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Pool Name *
                </label>
                <input
                  type="text"
                  name="name"
                  value={formData.name}
                  onChange={handleInputChange}
                  placeholder="e.g., Clean Water Initiative"
                  className="w-full px-4 py-3 bg-slate-900/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[#50C878]/50 focus:border-[#50C878] transition"
                  required
                  disabled={submissionState === "submitting"}
                />
              </div>

              {/* Description */}
              <div className="bg-slate-800/60 border border-slate-700/50 rounded-2xl p-6">
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Description * (max 500 characters)
                </label>
                <textarea
                  name="description"
                  value={formData.description}
                  onChange={handleInputChange}
                  placeholder="Describe your donation pool and its purpose..."
                  rows={4}
                  maxLength={500}
                  className="w-full px-4 py-3 bg-slate-900/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[#50C878]/50 focus:border-[#50C878] transition resize-none"
                  required
                  disabled={submissionState === "submitting"}
                />
                <p className="text-xs text-slate-500 mt-2">
                  {formData.description.length}/500 characters
                </p>
              </div>

              {/* External URL */}
              <div className="bg-slate-800/60 border border-slate-700/50 rounded-2xl p-6">
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  <LinkIcon size={16} className="inline mr-2" />
                  External URL * (max 200 characters)
                </label>
                <input
                  type="url"
                  name="externalUrl"
                  value={formData.externalUrl}
                  onChange={handleInputChange}
                  placeholder="https://example.com/your-cause"
                  maxLength={200}
                  className="w-full px-4 py-3 bg-slate-900/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[#50C878]/50 focus:border-[#50C878] transition"
                  required
                  disabled={submissionState === "submitting"}
                />
              </div>

              {/* Image Hash */}
              <div className="bg-slate-800/60 border border-slate-700/50 rounded-2xl p-6">
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  <ImageIcon size={16} className="inline mr-2" />
                  Image Hash * (max 100 characters)
                </label>
                <input
                  type="text"
                  name="imageHash"
                  value={formData.imageHash}
                  onChange={handleInputChange}
                  placeholder="Image hash or URL"
                  maxLength={100}
                  className="w-full px-4 py-3 bg-slate-900/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[#50C878]/50 focus:border-[#50C878] transition"
                  required
                  disabled={submissionState === "submitting"}
                />
              </div>

              {/* Target Amount */}
              <div className="bg-slate-800/60 border border-slate-700/50 rounded-2xl p-6">
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  <DollarSign size={16} className="inline mr-2" />
                  Target Amount (XLM) *
                </label>
                <input
                  type="number"
                  name="targetAmount"
                  value={formData.targetAmount}
                  onChange={handleInputChange}
                  placeholder="1000"
                  min="0"
                  step="0.0000001"
                  className="w-full px-4 py-3 bg-slate-900/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[#50C878]/50 focus:border-[#50C878] transition"
                  required
                  disabled={submissionState === "submitting"}
                />
              </div>

              {/* Duration */}
              <div className="bg-slate-800/60 border border-slate-700/50 rounded-2xl p-6">
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  <Calendar size={16} className="inline mr-2" />
                  Duration (Days) *
                </label>
                <input
                  type="number"
                  name="durationDays"
                  value={formData.durationDays}
                  onChange={handleInputChange}
                  placeholder="30"
                  min="1"
                  className="w-full px-4 py-3 bg-slate-900/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[#50C878]/50 focus:border-[#50C878] transition"
                  required
                  disabled={submissionState === "submitting"}
                />
                <p className="text-xs text-slate-500 mt-2">
                  The pool will be active for this many days
                </p>
              </div>

              {/* Error Message */}
              {errorMessage && submissionState === "idle" && (
                <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4">
                  <p className="text-red-400 text-sm">{errorMessage}</p>
                </div>
              )}

              {/* Submit Button */}
              <button
                type="submit"
                disabled={submissionState === "submitting" || !publicKey}
                className="w-full bg-gradient-to-r from-[#50C878] to-[#14B8A6] text-black px-8 py-4 rounded-xl font-semibold hover:shadow-lg hover:shadow-[#50C878]/20 transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
              >
                {submissionState === "submitting" ? (
                  <>
                    <Loader2 className="animate-spin" size={20} />
                    Creating Pool...
                  </>
                ) : (
                  "Create Pool"
                )}
              </button>
            </form>
          )}
        </div>
      </div>

      <Footer />
    </div>
  );
}
