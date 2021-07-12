import { distr, SPACE_BETWEEN } from "./distr";
import { Quadtree, pointWithin } from "./quadtree";
import uPlot from "uplot";

export function seriesBarsPlugin(pluginOpts) {
  let pxRatio;
  let font;
  const enableBarLabels = false;

  function setPxRatio() {
    pxRatio = devicePixelRatio;
    font = Math.round(10 * pxRatio) + "px Arial";
  }

  setPxRatio();

  window.addEventListener("dppxchange", setPxRatio);

  const labels = pluginOpts.labels;

  const ori = pluginOpts.ori;
  const dir = pluginOpts.dir;

  const groupWidth = 0.6;
  const groupDistr = SPACE_BETWEEN;

  const barWidth = 1.0;
  const barDistr = SPACE_BETWEEN;

  function walkTwo(yIdx, xCount, yCount, xDim, xDraw, yDraw) {
    distr(xCount, groupWidth, groupDistr, null, (ix, offPct, dimPct) => {
      let groupOffPx = xDim * offPct;
      let groupWidPx = xDim * dimPct;

      xDraw && xDraw(ix, groupOffPx, groupWidPx);

      yDraw &&
        distr(yCount, barWidth, barDistr, yIdx, (iy, offPct, dimPct) => {
          let barOffPx = groupWidPx * offPct;
          let barWidPx = groupWidPx * dimPct;

          yDraw(ix, groupOffPx + barOffPx, barWidPx);
        });
    });
  }

  function drawBars(u, sidx, i0, i1) {
    return uPlot.orient(
      u,
      sidx,
      (series, dataX, dataY, scaleX, scaleY, valToPosX, valToPosY, xOff, yOff, xDim, yDim, moveTo, lineTo, rect) => {
        const fill = new Path2D();
        const stroke = new Path2D();

        // test ori, text align, text baseline...x0, y0,m width, height

        let numGroups = dataX.length;
        let barsPerGroup = pluginOpts.bars.length;

        let y0Pos = valToPosY(0, scaleY, yDim, yOff);

        let strokeWidth = series.width || 0;

        const _dir = dir * (ori === 0 ? 1 : -1);

        walkTwo(sidx - 1, numGroups, barsPerGroup, xDim, null, (ix, x0, wid) => {
          let lft = Math.round(xOff + (_dir === 1 ? x0 : xDim - x0 - wid));
          let barWid = Math.round(wid);

          if (dataY[ix] != null) {
            let yPos = valToPosY(dataY[ix], scaleY, yDim, yOff);

            let btm = Math.round(Math.max(yPos, y0Pos));
            let top = Math.round(Math.min(yPos, y0Pos));
            let barHgt = btm - top;

            if (strokeWidth)
            {
              rect(stroke, lft + strokeWidth / 2, top + strokeWidth / 2, barWid - strokeWidth, barHgt - strokeWidth);
            }

            rect(fill, lft, top, barWid, barHgt);

            let x = ori === 0 ? Math.round(lft - xOff) : Math.round(top - yOff);
            let y = ori === 0 ? Math.round(top - yOff) : Math.round(lft - xOff);
            let w = ori === 0 ? barWid : barHgt;
            let h = ori === 0 ? barHgt : barWid;

            qt.add({ x, y, w, h, sidx: sidx, didx: ix });
          }
        });

        return {
          stroke,
          fill,
        };
      }
    );
  }

  function drawBarLabels(u, sidx, i0, i1) {
    if (!enableBarLabels) {
      return;
    }
    u.ctx.font = font;
    u.ctx.fillStyle = "black";

    uPlot.orient(
      u,
      sidx,
      (series, dataX, dataY, scaleX, scaleY, valToPosX, valToPosY, xOff, yOff, xDim, yDim, moveTo, lineTo, rect) => {
        let numGroups = dataX.length;
        let barsPerGroup = u.series.length - 1;

        const _dir = dir * (ori === 0 ? 1 : -1);

        walkTwo(sidx - 1, numGroups, barsPerGroup, xDim, null, (ix, x0, wid) => {
          let lft = Math.round(xOff + (_dir === 1 ? x0 : xDim - x0 - wid));
          let barWid = Math.round(wid);

          if (dataY[ix] != null) {
            let yPos = valToPosY(dataY[ix], scaleY, yDim, yOff);

            let x = ori === 0 ? Math.round(lft + barWid / 2) : Math.round(yPos);
            let y = ori === 0 ? Math.round(yPos) : Math.round(lft + barWid / 2);

            u.ctx.textAlign = ori === 0 ? "center" : dataY[ix] >= 0 ? "left" : "right";
            u.ctx.textBaseline = ori === 1 ? "middle" : dataY[ix] >= 0 ? "bottom" : "top";

            u.ctx.fillText(dataY[ix], x, y);
          }
        });
      }
    );
  }

  function range(u, dataMin, dataMax) {
    let [, max] = uPlot.rangeNum(0, dataMax, 0.05, true);
    return [0, max];
  }

  function lineRange(u, dataMin, dataMax) {
    return [-groupWidth / 2, dataMax + groupWidth/2];
  }

  let qt;
  let hovered = null;

  let barMark = document.createElement("div");
  barMark.classList.add("bar-mark");

  return {
    hooks: {
      init: (u) => {
        u.over.appendChild(barMark);
      },
      drawClear: (u) => {
        qt = qt || new Quadtree(0, 0, u.bbox.width, u.bbox.height);

        qt.clear();

        // force-clear the path cache to cause drawBars() to rebuild new quadtree
        u.series.forEach((s) => {
          s._paths = null;
        });
      },
      setCursor: (u) => {
        let found = null;
        let cx = u.cursor.left * pxRatio;
        let cy = u.cursor.top * pxRatio;

        qt.get(cx, cy, 1, 1, (o) => {
          if (pointWithin(cx, cy, o.x, o.y, o.x + o.w, o.y + o.h)) found = o;
        });

        if (found) {
          if (found !== hovered) {
            barMark.style.display = null;
            barMark.style.left = found.x / pxRatio + "px";
            barMark.style.top = found.y / pxRatio + "px";
            barMark.style.width = found.w / pxRatio + "px";
            barMark.style.height = found.h / pxRatio + "px";
            hovered = found;
          }
        } else if (hovered !== null) {
          hovered = null;
          barMark.style.display = "none";
        }
      },
    },
    opts: (u, opts) => {
      const yScaleOpts = {
        range,
        ori: ori === 0 ? 1 : 0,
      };

      uPlot.assign(opts, {
        select: { show: false },
        cursor: {
          x: false,
          y: false,
          points: { show: false },
        },
        scales: {
          x: {
            range: lineRange,
            time: false,
            distr: 2,
            ori,
            dir,
          },
          rend: yScaleOpts,
          size: yScaleOpts,
          mem: yScaleOpts,
          inter: yScaleOpts,
          toggle: yScaleOpts,
        },
      });

      if (ori === 1) {
        opts.padding = [0, null, 0, null];
      }

      uPlot.assign(opts.axes[0], {
        splits: (u, axisIdx) => {
          const dim = ori === 0 ? u.bbox.width : u.bbox.height;
          const _dir = dir * (ori === 0 ? 1 : -1);

          let splits = [];

          distr(u.data[0].length, groupWidth, groupDistr, null, (di, lftPct, widPct) => {
            let groupLftPx = (dim * lftPct) / pxRatio;
            let groupWidPx = (dim * widPct) / pxRatio;

            let groupCenterPx = groupLftPx + groupWidPx / 2;

            splits.push(u.posToVal(groupCenterPx, "x"));
          });

          return _dir === 1 ? splits : splits.reverse();
        },
        values: () => labels(),
        gap: 15,
        size: ori === 0 ? 40 : 150,
        labelSize: 20,
        grid: { show: false },
        ticks: { show: false },

        side: ori === 0 ? 2 : 3,
      });

      opts.series.forEach((s, i) => {
        if (pluginOpts.bars.includes(i)) {
          uPlot.assign(s, {
            paths: drawBars,
            points: {
              show: drawBarLabels,
            },
          });
        }
      });
    },
  };
}