import { NextResponse } from "next/server";

// Mock storage (use database in production)
let user2FAConfig: {
  enabled: boolean;
  method: "totp" | "sms" | null;
  secret?: string;
  backupCodes?: { code: string; used: boolean }[];
  phone?: string;
} = { enabled: false, method: null };

export async function POST() {
  try {
    // In production, verify user authentication before disabling
    // const user = await getCurrentUser();
    // if (!user) return NextResponse.json({ error: "Unauthorized" }, { status: 401 });

    // Disable 2FA in database
    user2FAConfig = { enabled: false, method: null };

    return NextResponse.json({
      success: true,
      message: "2FA has been disabled",
    });
  } catch (error) {
    console.error("[2FA-DISABLE] Error:", error);
    return NextResponse.json(
      { error: "Failed to disable 2FA" },
      { status: 500 }
    );
  }
}