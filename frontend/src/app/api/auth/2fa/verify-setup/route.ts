import { NextRequest, NextResponse } from "next/server";
import crypto from "crypto";
import base32Encode from "hi-base32";

// Verify TOTP code
function verifyTOTP(secret: string, code: string): boolean {
  // Simple TOTP verification (in production, use a proper TOTP library)
  // For demo purposes, accept any 6-digit code
  if (code.length !== 6) return false;
  
  // In production, use proper TOTP verification:
  // const counter = Math.floor(Date.now() / 30000);
  // const expected = generateTOTP(secret, counter);
  // return timingSafeEqual(Buffer.from(code), Buffer.from(expected));
  
  return /^\d{6}$/.test(code);
}

// Generate backup codes
function generateBackupCodes(count: number = 10): { code: string; used: boolean }[] {
  const codes: { code: string; used: boolean }[] = [];
  for (let i = 0; i < count; i++) {
    const code = crypto.randomBytes(4).toString("hex").toUpperCase();
    codes.push({ code: `${code.slice(0, 4)}-${code.slice(4)}`, used: false });
  }
  return codes;
}

// Mock storage (use database in production)
let user2FAConfig: {
  enabled: boolean;
  method: "totp" | "sms" | null;
  secret?: string;
  backupCodes?: { code: string; used: boolean }[];
  phone?: string;
} = { enabled: false, method: null };

let pending2FASetup: {
  secret?: string;
  method?: "totp" | "sms";
  expiresAt?: Date;
} = {};

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { code, method, phone } = body;

    if (!code) {
      return NextResponse.json(
        { error: "Verification code is required" },
        { status: 400 }
      );
    }

    // Check if setup is pending
    if (!pending2FASetup.method || !pending2FASetup.expiresAt || pending2FASetup.expiresAt < new Date()) {
      return NextResponse.json(
        { error: "No pending 2FA setup. Please start again." },
        { status: 400 }
      );
    }

    // Verify code
    let isValid = false;
    if (method === "totp") {
      isValid = verifyTOTP(pending2FASetup.secret || "", code);
    } else if (method === "sms") {
      // In production, verify SMS code
      isValid = code.length === 6 && /^\d{6}$/.test(code);
    }

    if (!isValid) {
      return NextResponse.json(
        { error: "Invalid verification code" },
        { status: 400 }
      );
    }

    // Generate backup codes
    const backupCodes = generateBackupCodes(10);

    // Store 2FA configuration (use database in production)
    user2FAConfig = {
      enabled: true,
      method: pending2FASetup.method,
      secret: pending2FASetup.secret,
      backupCodes,
      phone,
    };

    // Clear pending setup
    pending2FASetup = {};

    return NextResponse.json({
      success: true,
      backupCodes: backupCodes.map(c => c.code),
    });
  } catch (error) {
    console.error("[2FA-VERIFY-SETUP] Error:", error);
    return NextResponse.json(
      { error: "Failed to verify 2FA setup" },
      { status: 500 }
    );
  }
}