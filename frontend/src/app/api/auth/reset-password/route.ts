import { NextRequest, NextResponse } from "next/server";
import crypto from "crypto";

// In-memory token storage (must match forgot-password route)
const passwordResetTokens = new Map<string, { email: string; expiresAt: Date }>();

// Password hashing utility (use bcrypt in production with Node.js backend)
async function hashPassword(password: string): Promise<string> {
  // In production, use bcrypt or argon2
  // This is a simplified version using SHA-256 for demo
  const salt = crypto.randomBytes(16).toString("hex");
  const hash = crypto.pbkdf2Sync(password, salt, 1000, 64, "sha512").toString("hex");
  return `${salt}:${hash}`;
}

// Verify password hash
async function verifyPassword(password: string, storedHash: string): Promise<boolean> {
  const [salt, hash] = storedHash.split(":");
  const newHash = crypto.pbkdf2Sync(password, salt, 1000, 64, "sha512").toString("hex");
  return hash === newHash;
}

// Password strength validation
function validatePasswordStrength(password: string): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  if (password.length < 8) {
    errors.push("Password must be at least 8 characters");
  }
  if (!/[A-Z]/.test(password)) {
    errors.push("Password must contain at least one uppercase letter");
  }
  if (!/[a-z]/.test(password)) {
    errors.push("Password must contain at least one lowercase letter");
  }
  if (!/[0-9]/.test(password)) {
    errors.push("Password must contain at least one number");
  }
  if (!/[!@#$%^&*(),.?":{}|<>]/.test(password)) {
    errors.push("Password must contain at least one special character");
  }

  return { valid: errors.length === 0, errors };
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { token, password } = body;

    // Validate inputs
    if (!token) {
      return NextResponse.json(
        { error: "Token is required" },
        { status: 400 }
      );
    }

    if (!password) {
      return NextResponse.json(
        { error: "Password is required" },
        { status: 400 }
      );
    }

    // Validate password strength
    const passwordValidation = validatePasswordStrength(password);
    if (!passwordValidation.valid) {
      return NextResponse.json(
        { error: passwordValidation.errors.join(". ") },
        { status: 400 }
      );
    }

    // Check if token exists and is not expired
    const tokenData = passwordResetTokens.get(token);
    
    if (!tokenData) {
      return NextResponse.json(
        { error: "Invalid or expired reset token" },
        { status: 400 }
      );
    }

    if (tokenData.expiresAt < new Date()) {
      // Clean up expired token
      passwordResetTokens.delete(token);
      return NextResponse.json(
        { error: "Token has expired. Please request a new password reset." },
        { status: 400 }
      );
    }

    // Hash the new password securely
    const hashedPassword = await hashPassword(password);

    // Update user password in database (implement with actual database)
    // await db.user.update({
    //   where: { email: tokenData.email },
    //   data: { password: hashedPassword }
    // });

    console.log(`[RESET-PASSWORD] Password reset successful for: ${tokenData.email}`);

    // Invalidate the token after successful password reset
    passwordResetTokens.delete(token);

    // Send confirmation email (optional)
    // await sendPasswordChangedEmail(tokenData.email);

    return NextResponse.json({ 
      success: true, 
      message: "Password reset successful" 
    });
  } catch (error) {
    console.error("[RESET-PASSWORD] Error:", error);
    return NextResponse.json(
      { error: "An error occurred. Please try again." },
      { status: 500 }
    );
  }
}