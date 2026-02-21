import React from "react";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  variant?: "primary" | "secondary" | "outline";
  size?: "sm" | "md" | "lg";
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    { children, variant = "primary", size = "md", className = "", ...props },
    ref,
  ) => {
    const baseStyles =
      "font-semibold transition-all duration-300 hover:-translate-y-1 active:scale-95 rounded-lg";

    const variants = {
      primary: "bg-blue-600 hover:bg-blue-700 text-white active:shadow-[0_0_20px_rgba(37,99,235,0.6)]",
      secondary: "bg-white text-blue-600 hover:bg-slate-50",
      outline:
        "border-2 border-blue-600 text-blue-600 dark:text-blue-400 dark:border-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950",
    };

    const sizes = {
      sm: "px-4 py-2 text-sm",
      md: "px-6 py-2 text-base",
      lg: "px-8 py-3 text-lg",
    };

    return (
      <button
        ref={ref}
        className={`${baseStyles} ${variants[variant]} ${sizes[size]} ${className}`}
        {...props}
      >
        {children}
      </button>
    );
  },
);

Button.displayName = "Button";
