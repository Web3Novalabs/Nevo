import { NextResponse } from 'next/server';

type Submission = {
  id: string;
  name?: string;
  email: string;
  subject: string;
  message: string;
  autoReply: boolean;
  createdAt: string;
  ip: string;
};

const SUPPORT_EMAIL = process.env.SUPPORT_EMAIL || 'support@nevo.example';
const EMAIL_FROM = process.env.EMAIL_FROM || 'no-reply@nevo.example';
const STORE: Submission[] = [];
const RATE_LIMIT_MAP: Record<string, { count: number; firstTs: number }> = {};
const RATE_LIMIT_MAX = 5; // per hour per IP (mock)

function validateEmail(value: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
}

async function sendEmail(to: string, subject: string, text: string, html: string) {
  // eslint-disable-next-line no-console
  console.log(`[contact] Email queued to ${to}`);
  // eslint-disable-next-line no-console
  console.log({ from: EMAIL_FROM, to, subject, text, html });
  // TODO: Replace with a real email provider integration (SendGrid, SES, Mailgun, etc.)
}

export async function POST(req: Request) {
  try {
    const ip =
      (req.headers.get('x-forwarded-for') || req.headers.get('x-real-ip') || 'local') as string;
    const now = Date.now();
    const rl = RATE_LIMIT_MAP[ip] || { count: 0, firstTs: now };

    if (now - rl.firstTs > 1000 * 60 * 60) {
      rl.count = 0;
      rl.firstTs = now;
    }

    if (rl.count >= RATE_LIMIT_MAX) {
      return NextResponse.json({ error: 'Rate limit exceeded' }, { status: 429 });
    }

    const body = await req.json();
    const { name, email, subject, message, autoReply } = body || {};

    if (!email || !validateEmail(String(email))) {
      return NextResponse.json({ error: 'Invalid email' }, { status: 400 });
    }

    if (!subject || String(subject).trim().length < 3) {
      return NextResponse.json({ error: 'Invalid subject' }, { status: 400 });
    }

    if (!message || String(message).trim().length < 10) {
      return NextResponse.json({ error: 'Invalid message' }, { status: 400 });
    }

    const submission: Submission = {
      id: Math.random().toString(36).slice(2, 10),
      name: name ? String(name).trim() : undefined,
      email: String(email).trim(),
      subject: String(subject).trim(),
      message: String(message).trim(),
      autoReply: !!autoReply,
      createdAt: new Date().toISOString(),
      ip,
    };

    // Store submissions in an in-memory mock database for this implementation.
    STORE.push(submission);

    rl.count += 1;
    RATE_LIMIT_MAP[ip] = rl;

    const supportText = `New support request from ${submission.email}\n\nSubject: ${submission.subject}\n\nMessage:\n${submission.message}`;
    const supportHtml = `<p><strong>New support request from</strong> ${submission.email}</p><p><strong>Subject:</strong> ${submission.subject}</p><p><strong>Message:</strong></p><p>${submission.message.replace(/\n/g, '<br/>')}</p>`;
    await sendEmail(SUPPORT_EMAIL, `New contact inquiry: ${submission.subject}`, supportText, supportHtml);

    if (submission.autoReply) {
      const autoReplyText = `Thanks for reaching out! We received your message and will reply as soon as possible.`;
      const autoReplyHtml = `<p>Thanks for reaching out! We received your message and will reply as soon as possible.</p><p><strong>Subject:</strong> ${submission.subject}</p>`;
      await sendEmail(submission.email, 'Your Nevo support request was received', autoReplyText, autoReplyHtml);
    }

    return NextResponse.json({ ok: true, id: submission.id });
  } catch (err: any) {
    return NextResponse.json({ error: err?.message || 'Server error' }, { status: 500 });
  }
}

export async function GET() {
  return NextResponse.json({ submissions: STORE });
}
