import { NextRequest, NextResponse } from "next/server";
import crypto from "crypto";
import base32Encode from "hi-base32";

// Generate TOTP secret
function generateTOTPSecret(): string {
  const buffer = crypto.randomBytes(20);
  return base32Encode(buffer).replace(/=/g, "").toUpperCase();
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

// Generate QR Code URL (TOTP format)
function generateTOTPQRUrl(secret: string, email: string): string {
  const issuer = "Nevo";
  return `otpauth://totp/${issuer}:${email}?secret=${secret}&issuer=${issuer}&algorithm=SHA1&digits=6&period=30`;
}

// Mock storage (use database in production)
let pending2FASetup: {
  secret?: string;
  backupCodes?: { code: string; used: boolean }[];
  method?: "totp" | "sms";
  expiresAt?: Date;
} = {};

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { method } = body;

    if (!method || !["totp", "sms"].includes(method)) {
      return NextResponse.json(
        { error: "Invalid 2FA method" },
        { status: 400 }
      );
    }

    // In production, get user from session
    const userEmail = "user@example.com";

    if (method === "totp") {
      // Generate TOTP secret
      const secret = generateTOTPSecret();
      const qrCodeUrl = generateTOTPQRUrl(secret, userEmail);

      // Store temporarily (use database in production)
      pending2FASetup = {
        secret,
        method: "totp",
        expiresAt: new Date(Date.now() + 10 * 60 * 1000), // 10 minutes
      };

      return NextResponse.json({
        secret,
        qrCodeUrl,
      });
    } else if (method === "sms") {
      // In production, would send SMS and verify phone number
      const maskedPhone = "+1 (555) ***-****";
      
      pending2FASetup = {
        method: "sms",
        expiresAt: new Date(Date.now() + 10 * 60 * 1000),
      };

      return NextResponse.json({
        maskedPhone,
        message: "Verification code sent to your phone",
      });
    }

    return NextResponse.json(
      { error: "Invalid method" },
      { status: 400 }
    );
  } catch (error) {
    console.error("[2FA-SETUP] Error:", error);
    return NextResponse.json(
      { error: "Failed to setup 2FA" },
      { status: 500 }
    );
  }
}