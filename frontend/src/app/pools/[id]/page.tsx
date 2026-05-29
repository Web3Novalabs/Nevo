import Link from "next/link"

interface PoolDetailPageProps {
  params: Promise<{ id: string }>
}

export default async function PoolDetailPage({ params }: PoolDetailPageProps) {
  const { id } = await params

  return (
    <main className="min-h-screen bg-slate-950 px-4 py-16 text-white sm:px-6 lg:px-8">
      <div className="mx-auto w-full max-w-4xl rounded-2xl border border-slate-800 bg-slate-900/70 p-8">
        <p className="text-sm text-slate-400">Pool Detail</p>
        <h1 className="mt-2 text-3xl font-bold">Pool #{id}</h1>
        <p className="mt-4 text-slate-300">
          This page is ready for full pool detail integration. The listing cards now navigate here.
        </p>
        <Link
          href="/explore"
          className="mt-8 inline-flex rounded-lg bg-slate-800 px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-slate-700"
        >
          Back to pools
        </Link>
      </div>
    </main>
  )
}
