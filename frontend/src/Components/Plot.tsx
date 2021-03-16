import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

interface Props {
  options: uPlot.Options;
  data: uPlot.AlignedData;
}

function useWindowSize() {
  const [size, setSize] = useState([0, 0]);
  useLayoutEffect(() => {
    function updateSize() {
      setSize([window.innerWidth, window.innerHeight]);
    }
    window.addEventListener("resize", updateSize);
    updateSize();
    return () => window.removeEventListener("resize", updateSize);
  }, []);
  return size;
}

export const Plot: React.FC<Props> = ({ options, data }) => {
  const plotRef = useRef<HTMLDivElement>(null);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [width, height] = useWindowSize();
  useEffect(() => {
    const availableWidth = plotRef.current?.getBoundingClientRect().width;
    if (availableWidth) {
      options.width = availableWidth - 50;
    }
    new uPlot(options, data, plotRef.current as HTMLDivElement);
  }, [options, data]);
  return <div ref={plotRef} />;
};
