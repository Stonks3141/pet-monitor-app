import{S as y,i as g,s as E,k as h,q as S,a as b,l as d,m as _,r as q,h as c,c as w,n as l,Q as x,b as f,E as v,B as m}from"../../../chunks/index-a4c379a0.js";function C(n){let e,o,s,t,r,u;return{c(){e=h("a"),o=S("Settings"),s=b(),t=h("video"),r=h("source"),this.h()},l(a){e=d(a,"A",{href:!0,class:!0});var i=_(e);o=q(i,"Settings"),i.forEach(c),s=w(a),t=d(a,"VIDEO",{width:!0,height:!0});var p=_(t);r=d(p,"SOURCE",{src:!0,type:!0}),p.forEach(c),this.h()},h(){l(e,"href","/config"),l(e,"class","btn m-4 absolute top-0 right-0"),x(r.src,u="/stream.mp4")||l(r,"src",u),l(r,"type","video/mp4"),t.controls=!0,t.autoplay=!0,t.muted=!0,t.playsInline=!0,l(t,"width",n[0]),l(t,"height",n[1])},m(a,i){f(a,e,i),v(e,o),f(a,s,i),f(a,t,i),v(t,r)},p:m,i:m,o:m,d(a){a&&c(e),a&&c(s),a&&c(t)}}}function I(n,e,o){let{data:s}=e,[t,r]=s.resolution;return n.$$set=u=>{"data"in u&&o(2,s=u.data)},[t,r,s]}class k extends y{constructor(e){super(),g(this,e,I,C,E,{data:2})}}export{k as default};
