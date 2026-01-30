"use client";
import { useState } from "react";
import { ArrowUpRight, ChevronDown } from "lucide-react";
import Navigation from "../../components/Navigation";
import Footer from "../../components/Footer";

export default function ContactPage() {
  const [fullName, setFullName] = useState("");
  const [subject, setSubject] = useState("");
  const [message, setMessage] = useState("");
  const [email, setEmail] = useState("");
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState("");
  const isFormValid = fullName && email && subject && message;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    // Simple validation
    if (!fullName || !email || !subject || !message) {
      setError("Please fill in all fields.");
      setSuccess(false);
      return;
    }

    // Clear errors
    setError("");

    // Simulate successful submission
    console.log({ fullName, subject, message, email });

    setSuccess(true);

    // Optional: reset form
    setFullName("");
    setEmail("");
    setSubject("");
    setMessage("");
  };

  return (
    <>
      <Navigation />
      <div className=" bg-[#0F172A] ">
        <div className="w-full max-w-3xl mx-auto px-6 md:px-8 py-20 md:py-32  sticky overflow-y-hidden">
          <div className="bg-[#1E293B] rounded-2xl p-6 md:p-8">
            <div className="mb-12 text-left">
              <h1 className="text-white text-2xl md:text-3xl font-light mb-4">
                Contact Support
              </h1>
              <p className="text-[#50C878] text-sm font-light">
                We are here if you need help or clarity on things concerning Nevo
              </p>
            </div>

            <form onSubmit={handleSubmit} className="space-y-6">
              {error && (
                <p className="mb-4 text-sm text-red-400 font-light">{error}</p>
              )}

              {success && (
                <p className="mb-4 text-sm text-[#50C878] font-light">
                  Your message has been sent successfully 🎉
                </p>
              )}

              {/* Full Name Field */}
              <div>
                <label
                  htmlFor="fullName"
                  className="block text-white/ text-sm font-light mb-3"
                >
                  Full Name
                </label>
                <input
                  type="text"
                  id="fullName"
                  value={fullName}
                  onChange={(e) => setFullName(e.target.value)}
                  placeholder="John Doe"
                  className="w-full px-4 py-3.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder:text-white/30 focus:outline-none focus:border-cyan-400/50 focus:bg-white/[0.07] transition-all text-sm font-light"
                />
              </div>
              {/* Email Field */}
              <div>
                <label
                  htmlFor="email"
                  className="block text-white/ text-sm font-light mb-3"
                >
                  Email
                </label>
                <input
                  type="email"
                  id="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="Enter Email address"
                  className="w-full px-4 py-3.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder:text-white/30 focus:outline-none focus:border-cyan-400/50 focus:bg-white/[0.07] transition-all text-sm font-light"
                />
              </div>

              {/* Subject Field */}
              <div>
                <label
                  htmlFor="subject"
                  className="block text-white/70 text-sm font-light mb-3"
                >
                  Subject
                </label>
                <div className="relative">
                  <select
                    id="subject"
                    value={subject}
                    onChange={(e) => setSubject(e.target.value)}
                    className="w-full px-4 py-3.5 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-cyan-400/50 focus:bg-white/[0.07] transition-all appearance-none text-sm font-light cursor-pointer"
                  >
                    <option value="">Select A Subject</option>
                    <option value="technical">Technical Support</option>
                    <option value="general">General Inquiry</option>
                  </select>
                  <ChevronDown className="absolute right-4 top-1/2 -translate-y-1/2 w-5 h-5 text-white/30 pointer-events-none" />
                </div>
              </div>

              {/* Message Field */}
              <div>
                <label
                  htmlFor="message"
                  className="block text-white/70 text-sm font-light mb-3"
                >
                  Message
                </label>
                <textarea
                  id="message"
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  placeholder="Text"
                  rows={8}
                  className="w-full px-4 py-3.5 bg-white/5 border border-white/10 rounded-lg text-white placeholder:text-white/30 focus:outline-none focus:border-cyan-400/50 focus:bg-white/[0.07] transition-all resize-none text-sm font-light"
                />
              </div>

              {/* Submit Button */}
              <div className="pt-4 flex justify-center">
                <button
                  type="submit"
                  disabled={!isFormValid}
                  className={`flex items-center justify-center gap-2 px-8 py-3 rounded-t-lg rounded-b-[18px] font-semibold transition-all duration-300
    ${
      isFormValid
        ? "bg-[#50C878] text-[#0F172A] cursor-pointer"
        : "bg-[#50C878]/60 text-[#0F172A]/70 cursor-not-allowed"
    }`}
                >
                  SEND MESSAGE
                  <ArrowUpRight size={16} aria-hidden={true} />
                </button>
              </div>
            </form>
          </div>
        </div>

        <Footer />
      </div>
    </>
  );
}
