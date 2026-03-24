"use client"

import { useScroll, motion, useSpring } from "framer-motion";

const PageProgress = () => {
  const { scrollYProgress } = useScroll();

  const scaleX = useSpring(scrollYProgress, {
    stiffness: 100,
    damping: 30,
    restDelta: 0.001,
  });
  return (
    <div>
      <motion.div
        className=" fixed top-0 left-0 right-0 bg-[#50C878] origin-[0%] h-1.25 z-99"
        style={{ scaleX }}
      />
    </div>
  );
};

export default PageProgress;
