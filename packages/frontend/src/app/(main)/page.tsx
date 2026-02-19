import { Navigation } from "@/components/layout/Navigation";
import { LandingPage } from "@/components/landing/LandingPage";
import { getStargazersCount } from "@/lib/github";

export default async function HomePage() {
  const stargazersCount = await getStargazersCount("junhoyeo/tokscale");

  return (
    <>
      <Navigation />
      <LandingPage stargazersCount={stargazersCount} />
    </>
  );
}
