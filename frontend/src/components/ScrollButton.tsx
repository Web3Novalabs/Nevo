"use client"

import { useEffect, useState } from "react";
import { ChevronsUp } from "lucide-react";

const ScrollButton = () => {

    const [isVisible, setIsVisible] = useState(false);

    useEffect(() => {
        const toggleVisibility = () => {
            if (window.scrollY > 200) {
                setIsVisible(true);
            } else {
                setIsVisible(false);
            }
        };

        window.addEventListener('scroll', toggleVisibility);

        return () => {
            window.removeEventListener('scroll', toggleVisibility);
        };
    }, []);

    const scrollToTop = () => {
        window.scrollTo({
            top: 0,
            behavior: 'smooth'
        });
    };

    return (
        <div className="fixed bottom-8 right-8 z-99">
            {
                isVisible && (<button onClick={scrollToTop} className="px-3.5 py-3.5 bg-[#50C878] duration-200 transition-all hover:bg-lilac text-slate-900 md:text-2xl text-base rounded-[3px] shadow-lg">
                    <ChevronsUp className="animate-pulse" />
                </button>)
            }
        </div>


    )
}

export default ScrollButton