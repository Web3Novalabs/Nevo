import { NextRequest, NextResponse } from "next/server";

// In-memory token storage (must match forgot-password route)
const passwordResetTokens = new Map<string, { email: string; expiresAt: Date }>();

export async function GET(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const token = searchParams.get("token");

    if (!token) {
      return NextResponse.json(
        { valid: false, error: "Token is required" },
        { status: 400 }
      );
    }

    // Check if token exists and is not expired
    const tokenData = passwordResetTokens.get(token);
    
    if (!tokenData) {
      return NextResponse.json(
        { valid: false, error: "Invalid token" },
        { status: 400 }
      );
    }

    if (tokenData.expiresAt < new Date()) {
      // Clean up expired token
      passwordResetTokens.delete(token);
      return NextResponse.json(
        { valid: false, error: "Token has expired" },
        { status: 400 }
      );
    }

    return NextResponse.json({ valid: true, email: tokenData.email });
  } catch (error) {
    console.error("[VALIDATE-TOKEN] Error:", error);
    return NextResponse.json(
      { valid: false, error: "An error occurred" },
      { status: 500 }
    );
  }
}