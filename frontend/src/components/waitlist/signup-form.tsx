"use client";

import { useState, FormEvent, ChangeEvent } from "react";
import { CheckCircle, AlertCircle } from "lucide-react";

interface FormData {
  email: string;
  firstName: string;
  lastName: string;
  privacyAccepted: boolean;
}

interface FormErrors {
  email?: string;
  firstName?: string;
  privacyAccepted?: string;
}

export const WaitlistSignupForm = () => {
  const [formData, setFormData] = useState<FormData>({
    email: "",
    firstName: "",
    lastName: "",
    privacyAccepted: false,
  });

  const [errors, setErrors] = useState<FormErrors>({});
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitted, setIsSubmitted] = useState(false);
  const [focusedField, setFocusedField] = useState<string | null>(null);

  const validateEmail = (email: string): boolean => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  };

  const validateForm = (): boolean => {
    const newErrors: FormErrors = {};

    if (!formData.email.trim()) {
      newErrors.email = "Email is required";
    } else if (!validateEmail(formData.email)) {
      newErrors.email = "Please enter a valid email address";
    }

    if (!formData.firstName.trim()) {
      newErrors.firstName = "First name is required";
    }

    if (!formData.privacyAccepted) {
      newErrors.privacyAccepted =
        "You must accept the privacy policy to continue";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const { name, value, type, checked } = e.target;
    setFormData((prev) => ({
      ...prev,
      [name]: type === "checkbox" ? checked : value,
    }));

    if (errors[name as keyof FormErrors]) {
      setErrors((prev) => ({
        ...prev,
        [name]: undefined,
      }));
    }
  };

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setIsLoading(true);

    try {
      // Simulate API call
      await new Promise((resolve) => setTimeout(resolve, 1500));

      setIsSubmitted(true);
      setFormData({
        email: "",
        firstName: "",
        lastName: "",
        privacyAccepted: false,
      });

      // Reset success state after 5 seconds
      setTimeout(() => {
        setIsSubmitted(false);
      }, 5000);
    } catch {
      setErrors({
        email: "Failed to join waitlist. Please try again.",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleFocus = (fieldName: string) => {
    setFocusedField(fieldName);
  };

  const handleBlur = () => {
    setFocusedField(null);
  };

  if (isSubmitted) {
    return (
      <div className="flex items-center justify-center min-h-[400px] px-4">
        <div className="text-center">
          <div className="flex justify-center mb-6">
            <CheckCircle className="w-16 h-16 text-green-500" />
          </div>
          <h3 className="text-2xl sm:text-3xl font-bold text-slate-900 dark:text-white mb-2">
            Welcome to the Waitlist!
          </h3>
          <p className="text-slate-600 dark:text-slate-300 mb-2">
            Thanks for joining, {formData.firstName}!
          </p>
          <p className="text-sm text-slate-500 dark:text-slate-400">
            We&apos;ll send you updates to {formData.email} soon.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="w-full max-w-md mx-auto px-4">
      <form onSubmit={handleSubmit} className="space-y-5">
        {/* Email Field */}
        <div>
          <label
            htmlFor="email"
            className="block text-sm font-semibold text-slate-900 dark:text-white mb-2"
          >
            Email Address <span className="text-red-500">*</span>
          </label>
          <input
            id="email"
            type="email"
            name="email"
            value={formData.email}
            onChange={handleChange}
            onFocus={() => handleFocus("email")}
            onBlur={handleBlur}
            disabled={isLoading}
            placeholder="you@example.com"
            className={`w-full px-4 py-3 rounded-lg border-2 transition-all duration-200 outline-none ${focusedField === "email"
                ? "border-blue-500 bg-blue-50 dark:bg-blue-950/30"
                : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900"
              } ${errors.email ? "border-red-500 bg-red-50 dark:bg-red-950/30" : ""
              } text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-slate-500 disabled:opacity-50 disabled:cursor-not-allowed`}
            aria-label="Email address"
            required
          />
          {errors.email && (
            <div className="flex items-center gap-2 mt-2 text-sm text-red-600 dark:text-red-400">
              <AlertCircle size={16} />
              <span>{errors.email}</span>
            </div>
          )}
        </div>

        {/* First Name Field */}
        <div>
          <label
            htmlFor="firstName"
            className="block text-sm font-semibold text-slate-900 dark:text-white mb-2"
          >
            First Name <span className="text-red-500">*</span>
          </label>
          <input
            id="firstName"
            type="text"
            name="firstName"
            value={formData.firstName}
            onChange={handleChange}
            onFocus={() => handleFocus("firstName")}
            onBlur={handleBlur}
            disabled={isLoading}
            placeholder="John"
            className={`w-full px-4 py-3 rounded-lg border-2 transition-all duration-200 outline-none ${focusedField === "firstName"
                ? "border-blue-500 bg-blue-50 dark:bg-blue-950/30"
                : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900"
              } ${errors.firstName
                ? "border-red-500 bg-red-50 dark:bg-red-950/30"
                : ""
              } text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-slate-500 disabled:opacity-50 disabled:cursor-not-allowed`}
            aria-label="First name"
            required
          />
          {errors.firstName && (
            <div className="flex items-center gap-2 mt-2 text-sm text-red-600 dark:text-red-400">
              <AlertCircle size={16} />
              <span>{errors.firstName}</span>
            </div>
          )}
        </div>

        {/* Last Name Field */}
        <div>
          <label
            htmlFor="lastName"
            className="block text-sm font-semibold text-slate-900 dark:text-white mb-2"
          >
            Last Name <span className="text-slate-400">(optional)</span>
          </label>
          <input
            id="lastName"
            type="text"
            name="lastName"
            value={formData.lastName}
            onChange={handleChange}
            onFocus={() => handleFocus("lastName")}
            onBlur={handleBlur}
            disabled={isLoading}
            placeholder="Doe"
            className={`w-full px-4 py-3 rounded-lg border-2 transition-all duration-200 outline-none ${focusedField === "lastName"
                ? "border-blue-500 bg-blue-50 dark:bg-blue-950/30"
                : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900"
              } text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-slate-500 disabled:opacity-50 disabled:cursor-not-allowed`}
            aria-label="Last name"
          />
        </div>

        {/* Privacy Policy Checkbox */}
        <div className="space-y-3">
          <div className="flex items-start gap-3">
            <input
              id="privacy"
              type="checkbox"
              name="privacyAccepted"
              checked={formData.privacyAccepted}
              onChange={handleChange}
              disabled={isLoading}
              className={`mt-1 w-5 h-5 rounded border-2 ${errors.privacyAccepted
                  ? "border-red-500 accent-red-600"
                  : "border-slate-300 dark:border-slate-600 accent-blue-600"
                } cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed transition-colors`}
              aria-label="Accept privacy policy"
            />
            <label
              htmlFor="privacy"
              className="text-sm text-slate-600 dark:text-slate-300"
            >
              I agree to the{" "}
              <a
                href="/privacy-policy"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline font-semibold"
              >
                Privacy Policy
              </a>{" "}
              and{" "}
              <a
                href="/terms-conditions"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline font-semibold"
              >
                Terms &amp; Conditions
              </a>
              <span className="text-red-500">*</span>
            </label>
          </div>
          {errors.privacyAccepted && (
            <div className="flex items-center gap-2 text-sm text-red-600 dark:text-red-400">
              <AlertCircle size={16} />
              <span>{errors.privacyAccepted}</span>
            </div>
          )}
        </div>

        {/* Submit Button */}
        <button
          type="submit"
          disabled={isLoading}
          className={`w-full py-3 px-4 rounded-lg font-semibold transition-all duration-200 transform ${isLoading
              ? "bg-slate-400 dark:bg-slate-600 cursor-not-allowed opacity-75"
              : "bg-blue-600 hover:bg-blue-700 active:scale-95 text-white dark:bg-blue-700 dark:hover:bg-blue-600"
            }`}
        >
          {isLoading ? (
            <span className="flex items-center justify-center gap-2">
              <svg
                className="w-4 h-4 animate-spin"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  className="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="4"
                />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              Joining...
            </span>
          ) : (
            "Join the Waitlist"
          )}
        </button>

        {/* General Error Message */}
        {errors.email && errors.email.includes("Failed") && (
          <div className="p-3 bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-900 rounded-lg flex items-center gap-2">
            <AlertCircle className="w-4 h-4 text-red-600 dark:text-red-400 flex-shrink-0" />
            <p className="text-sm text-red-700 dark:text-red-300">
              {errors.email}
            </p>
          </div>
        )}
      </form>

      {/* Trust Elements */}
      <p className="text-xs text-slate-500 dark:text-slate-400 text-center mt-6">
        🔒 Your data is secure and won&apos;t be shared
      </p>
    </div>
  );
};
