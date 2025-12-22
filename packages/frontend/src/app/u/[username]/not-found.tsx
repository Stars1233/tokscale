import Link from "next/link";
import { Navigation } from "@/components/layout/Navigation";
import { Footer } from "@/components/layout/Footer";

export default function ProfileNotFound() {
  return (
    <div className="min-h-screen flex flex-col" style={{ backgroundColor: "#10121C" }}>
      <Navigation />
      <main className="flex-1 max-w-[800px] mx-auto px-6 py-10 w-full">
        <div className="text-center py-20">
          <h1 className="text-2xl font-bold text-white mb-2">
            User Not Found
          </h1>
          <p className="mb-6" style={{ color: "#696969" }}>
            This user doesn&apos;t exist or hasn&apos;t submitted any data yet.
          </p>
          <Link
            href="/"
            className="inline-flex items-center px-4 py-2 text-white font-medium rounded-lg transition-colors hover:opacity-90"
            style={{ backgroundColor: "#53d1f3" }}
          >
            Back to Leaderboard
          </Link>
        </div>
      </main>
      <Footer />
    </div>
  );
}
