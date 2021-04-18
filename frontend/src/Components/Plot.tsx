import React, { useEffect, useLayoutEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

interface Props {
  options: uPlot.Options;
  data: uPlot.AlignedData;
}

export const Plot: React.FC<Props> = ({ options, data }) => {
  const plotDivRef = useRef<HTMLDivElement>(null);
  const plotInstanceRef = useRef<uPlot | null>(null);

  // Create uPlot instance
  useEffect(() => {
    if (plotDivRef.current !== null && plotInstanceRef.current === null) {
      console.debug("[uplot] create instance");
      const availableSize = plotDivRef.current?.getBoundingClientRect();
      if (availableSize) {
        options.width = availableSize.width;
      }
      const uPlotInstance = new uPlot(options, data, plotDivRef.current);
      plotInstanceRef.current = uPlotInstance;

      return () => {
        (plotInstanceRef.current as uPlot).destroy();
        plotInstanceRef.current = null;
      };
    }
  }, [data, options]);

  // Subscribe to resize events
  useLayoutEffect(() => {
    function updateSize() {
      const availableSize = plotDivRef.current?.getBoundingClientRect();
      if (availableSize) {
        plotInstanceRef.current?.setSize({ height: plotInstanceRef.current?.height, width: availableSize.width });
      }
    }

    window.addEventListener("resize", updateSize);
    return () => window.removeEventListener("resize", updateSize);
  }, []);

  return <div ref={plotDivRef} />;
};
