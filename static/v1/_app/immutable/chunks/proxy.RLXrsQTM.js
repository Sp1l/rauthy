import{S as p,R as S,T,V as l,W as U,b as f,X as R,U as s,g as h,j as A,Y as E,Z as I,_ as L}from"./index-client.BTDbhFDr.js";function g(i,d=null,V){if(typeof i!="object"||i===null||p in i)return i;const O=I(i);if(O!==S&&O!==T)return i;var n=new Map,c=L(i),m=l(0);c&&n.set("length",l(i.length));var b;return new Proxy(i,{defineProperty(v,r,e){(!("value"in e)||e.configurable===!1||e.enumerable===!1||e.writable===!1)&&U();var t=n.get(r);return t===void 0?(t=l(e.value),n.set(r,t)):f(t,g(e.value,b)),!0},deleteProperty(v,r){var e=n.get(r);if(e===void 0)r in v&&n.set(r,l(s));else{if(c&&typeof r=="string"){var t=n.get("length"),a=Number(r);Number.isInteger(a)&&a<t.v&&f(t,a)}f(e,s),K(m)}return!0},get(v,r,e){var u;if(r===p)return i;var t=n.get(r),a=r in v;if(t===void 0&&(!a||(u=R(v,r))!=null&&u.writable)&&(t=l(g(a?v[r]:s,b)),n.set(r,t)),t!==void 0){var o=h(t);return o===s?void 0:o}return Reflect.get(v,r,e)},getOwnPropertyDescriptor(v,r){var e=Reflect.getOwnPropertyDescriptor(v,r);if(e&&"value"in e){var t=n.get(r);t&&(e.value=h(t))}else if(e===void 0){var a=n.get(r),o=a==null?void 0:a.v;if(a!==void 0&&o!==s)return{enumerable:!0,configurable:!0,value:o,writable:!0}}return e},has(v,r){var o;if(r===p)return!0;var e=n.get(r),t=e!==void 0&&e.v!==s||Reflect.has(v,r);if(e!==void 0||A!==null&&(!t||(o=R(v,r))!=null&&o.writable)){e===void 0&&(e=l(t?g(v[r],b):s),n.set(r,e));var a=h(e);if(a===s)return!1}return t},set(v,r,e,t){var D;var a=n.get(r),o=r in v;if(c&&r==="length")for(var u=e;u<a.v;u+=1){var y=n.get(u+"");y!==void 0?f(y,s):u in v&&(y=l(s),n.set(u+"",y))}a===void 0?(!o||(D=R(v,r))!=null&&D.writable)&&(a=l(void 0),f(a,g(e,b)),n.set(r,a)):(o=a.v!==s,f(a,g(e,b)));var w=Reflect.getOwnPropertyDescriptor(v,r);if(w!=null&&w.set&&w.set.call(t,e),!o){if(c&&typeof r=="string"){var j=n.get("length"),P=Number(r);Number.isInteger(P)&&P>=j.v&&f(j,P+1)}K(m)}return!0},ownKeys(v){h(m);var r=Reflect.ownKeys(v).filter(a=>{var o=n.get(a);return o===void 0||o.v!==s});for(var[e,t]of n)t.v!==s&&!(e in v)&&r.push(e);return r},setPrototypeOf(){E()}})}function K(i,d=1){f(i,i.v+d)}function N(i){return i!==null&&typeof i=="object"&&p in i?i[p]:i}function M(i,d){return Object.is(N(i),N(d))}export{M as i,g as p};
