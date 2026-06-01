import { NextResponse } from "next/server";

// Mock user data (use database in production)
const mockUser2FA = {
  enabled: false,
  method: null as "totp" | "sms" | null,
};

export async function GET() {
  try {
    // In production, fetch from database based on authenticated user
    // const user = await getCurrentUser();
    // const twoFA = await db.twoFactorAuth.findUnique({ where: { userId: user.id } });
    
    return NextResponse.json({
      enabled: mockUser2FA.enabled,
      method: mockUser2FA.method,
    });
  } catch (error) {
    console.error("[2FA-STATUS] Error:", error);
    return NextResponse.json(
      { error: "Failed to fetch 2FA status" },
      { status: 500 }
    );
  }
}