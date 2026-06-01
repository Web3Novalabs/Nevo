import { NextRequest, NextResponse } from "next/server";
import crypto from "crypto";

// In-memory token storage (use database in production)
const passwordResetTokens = new Map<string, { email: string; expiresAt: Date }>();

// Email configuration (configure with actual email service)
const EMAIL_FROM = process.env.EMAIL_FROM || "noreply@nevo.com";
const BASE_URL = process.env.NEXT_PUBLIC_BASE_URL || "http://localhost:3000";

// Generate secure token
function generateResetToken(): string {
  return crypto.randomBytes(32).toString("hex");
}

// Send reset email (implement with actual email service)
async function sendPasswordResetEmail(email: string, token: string) {
  const resetLink = `${BASE_URL}/reset-password/${token}`;
  
  // Email template
  const emailHtml = `
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="utf-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
    </head>
    <body style="margin: 0; padding: 0; background-color: #f8fafc; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
      <table width="100%" cellpadding="0" cellspacing="0" style="background-color: #f8fafc; padding: 40px 20px;">
        <tr>
          <td align="center">
            <table width="100%" cellpadding="0" cellspacing="0" style="max-width: 600px; background-color: #ffffff; border-radius: 12px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);">
              <!-- Header -->
              <tr>
                <td style="padding: 40px 40px 20px; text-align: center;">
                  <h1 style="margin: 0; color: #1e293b; font-size: 28px; font-weight: 700;">Nevo</h1>
                  <p style="margin: 8px 0 0; color: #64748b; font-size: 14px;">Crowdfunding Platform</p>
                </td>
              </tr>
              
              <!-- Content -->
              <tr>
                <td style="padding: 20px 40px;">
                  <h2 style="margin: 0 0 16px; color: #1e293b; font-size: 20px; font-weight: 600;">Reset Your Password</h2>
                  <p style="margin: 0 0 16px; color: #475569; font-size: 15px; line-height: 1.6;">
                    We received a request to reset your password. Click the button below to create a new password:
                  </p>
                  
                  <!-- CTA Button -->
                  <table width="100%" cellpadding="0" cellspacing="0">
                    <tr>
                      <td align="center" style="padding: 24px 0;">
                        <a href="${resetLink}" style="display: inline-block; padding: 14px 32px; background-color: #4f46e5; color: #ffffff; text-decoration: none; font-weight: 600; font-size: 15px; border-radius: 8px;">
                          Reset Password
                        </a>
                      </td>
                    </tr>
                  </table>
                  
                  <p style="margin: 0 0 16px; color: #475569; font-size: 14px; line-height: 1.6;">
                    This link will expire in <strong>1 hour</strong> for security reasons.
                  </p>
                  
                  <p style="margin: 0; color: #94a3b8; font-size: 13px; line-height: 1.6;">
                    If you didn't request this password reset, please ignore this email or contact support if you have concerns.
                  </p>
                </td>
              </tr>
              
              <!-- Footer -->
              <tr>
                <td style="padding: 24px 40px; border-top: 1px solid #e2e8f0; text-align: center;">
                  <p style="margin: 0; color: #94a3b8; font-size: 12px;">
                    © ${new Date().getFullYear()} Nevo. All rights reserved.
                  </p>
                  <p style="margin: 8px 0 0; color: #94a3b8; font-size: 12px;">
                    This email was sent to <span style="color: #4f46e5;">${email}</span>
                  </p>
                </td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    </body>
    </html>
  `;

  // In production, integrate with email service (SendGrid, AWS SES, etc.)
  console.log(`[EMAIL] Sending password reset to: ${email}`);
  console.log(`[EMAIL] Reset link: ${resetLink}`);
  console.log(`[EMAIL] HTML preview:`, emailHtml.substring(0, 200) + "...");
  
  // Example with SendGrid:
  // await sgMail.send({ to: email, from: EMAIL_FROM, subject: "Reset Your Nevo Password", html: emailHtml });
  
  return true;
}

// Clean up expired tokens
function cleanupExpiredTokens() {
  const now = new Date();
  for (const [token, data] of passwordResetTokens.entries()) {
    if (data.expiresAt < now) {
      passwordResetTokens.delete(token);
    }
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { email } = body;

    // Validate email
    if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      return NextResponse.json(
        { error: "Please provide a valid email address" },
        { status: 400 }
      );
    }

    // Check if user exists (implement with actual database)
    // const user = await db.user.findUnique({ where: { email } });
    // if (!user) {
    //   // Don't reveal if email exists for security
    //   return NextResponse.json({ success: true });
    // }

    // Generate reset token
    const token = generateResetToken();
    const expiresAt = new Date(Date.now() + 60 * 60 * 1000); // 1 hour

    // Store token
    passwordResetTokens.set(token, { email, expiresAt });

    // Send reset email
    await sendPasswordResetEmail(email, token);

    // Clean up expired tokens periodically
    cleanupExpiredTokens();

    // Always return success to prevent email enumeration
    return NextResponse.json({ success: true });
  } catch (error) {
    console.error("[FORGOT-PASSWORD] Error:", error);
    return NextResponse.json(
      { error: "An error occurred. Please try again." },
      { status: 500 }
    );
  }
}