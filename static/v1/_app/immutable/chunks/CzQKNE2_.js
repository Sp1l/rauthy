import { n as U, a as h, t as _ } from "./BxmJRzoY.js";
import { t as k, p as V, j as N, a4 as X, a5 as Y, s as S, a as Z, c as l, k as t, l as p, r as v } from "./w0HvPX0p.js";
import { d as $, e as z } from "./BzP2S3Z_.js";
import { i as M } from "./iO9_dPNE.js";
import { s as K } from "./DM69BKKN.js";
import { s as d, a as P, b as tt } from "./BdbQ6g_y.js";
import { b as et } from "./CJLp5kwW.js";
import { p as c } from "./C6GSeq7M.js";
import { g as at } from "./B21bTIl7.js";
import { B as ot } from "./C8YTstTD.js";
var st = U('<svg fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" d="M7.5 21 3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5"></path></svg>');
function it(b, e) {
  let f = c(e, "opacity", 3, 0.9), q = c(e, "width", 3, "1.5rem"), y = c(e, "color", 3, "currentColor");
  var n = st();
  d(n, "stroke-width", 2), k(() => {
    d(n, "stroke", y()), d(n, "width", q()), d(n, "opacity", f());
  }), h(b, n);
}
function rt(b, e, f) {
  t(e) && p(f, true);
}
var lt = _('<div class="btn svelte-fpq55v"><!></div>'), vt = _('<div class="btnCollapsed svelte-fpq55v"><!></div>'), dt = _('<div class="absolute svelte-fpq55v"><div><!></div></div>'), nt = _("<ul><!></ul>"), ct = _('<div class="navSub svelte-fpq55v"><!> <nav class="svelte-fpq55v"><!> <div class="svelte-fpq55v"><!> <!></div></nav></div>');
function qt(b, e) {
  V(e, true);
  let f = c(e, "width", 3, "min(25rem, 100dvw)"), q = c(e, "paddingTop", 3, "4.5rem"), y = c(e, "collapseButtonThreshold", 3, 800), n = c(e, "thresholdNavSub", 3, 500);
  const j = at();
  let u = N(void 0), w = X(() => !!(t(u) && t(u) < n())), s = N(false), T = N(false);
  Y(() => {
    t(u) && p(s, t(w), true);
  });
  function R() {
    p(s, !t(s));
  }
  function D() {
    t(u) && t(u) < y() && p(T, true);
  }
  function E() {
    p(T, false);
  }
  var B = ct();
  {
    const g = (o) => {
      ot(o, { ariaControls: j, invisible: true, onclick: R, children: (a, r) => {
        var m = lt(), L = l(m);
        it(L, { width: "1.4rem" }), v(m), h(a, m);
      }, $$slots: { default: true } });
    };
    var x = l(B);
    {
      var F = (o) => {
        var a = vt(), r = l(a);
        g(r), v(a), h(o, a);
      };
      M(x, (o) => {
        (t(s) || t(w)) && o(F);
      });
    }
    var i = S(x, 2);
    d(i, "id", j), i.__click = [rt, w, s];
    let I;
    var A = l(i);
    {
      var G = (o) => {
        var a = dt(), r = l(a);
        let m;
        var L = l(r);
        g(L), v(r), v(a), k((Q) => m = tt(r, 1, "iconHover svelte-fpq55v", null, m, Q), [() => ({ hoverLeft: t(s) })]), h(o, a);
      };
      M(A, (o) => {
        t(T) && !(t(s) || t(w)) && o(G);
      });
    }
    var C = S(A, 2);
    let W;
    var H = l(C);
    K(H, () => e.children);
    var J = S(H, 2);
    {
      var O = (o) => {
        var a = nt(), r = l(a);
        K(r, () => e.buttonTiles), v(a), k(() => d(a, "aria-controls", e.buttonTilesAriaControls)), h(o, a);
      };
      M(J, (o) => {
        e.buttonTiles && o(O);
      });
    }
    v(C), v(i), v(B), k(() => {
      d(i, "aria-hidden", t(s)), d(i, "data-collapsed", t(s)), I = P(i, "", I, { width: t(s) ? 0 : f(), "min-width": t(s) ? 0 : f() }), W = P(C, "", W, { "padding-top": q() });
    }), z("mouseenter", i, D), z("mouseleave", i, E);
  }
  et("innerWidth", (g) => p(u, g, true)), h(b, B), Z();
}
$(["click"]);
export {
  qt as N
};
