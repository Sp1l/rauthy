import{f as s}from"./helpers.DwhnXD2f.js";const e={json:{"Content-Type":"application/json",Accept:"application/json"},form:{"Content-type":"application/x-www-form-urlencoded",Accept:"application/json"}};function a(){return{...e.json,"csrf-token":s()}}async function o(){return await fetch("/auth/v1/version",{method:"GET",headers:e.json})}async function r(){return await fetch("/auth/v1/auth_check_admin",{method:"GET",headers:e.json})}async function h(t){return await fetch("/auth/v1/oidc/token",{method:"POST",headers:e.form,body:t})}async function c(){return await fetch("/auth/v1/oidc/sessioninfo",{method:"GET",headers:e.json})}async function i(t){return await fetch("/auth/v1/oidc/sessioninfo/xsrf",{method:"GET",headers:{...e.json,Authorization:`Bearer ${t}`}})}async function u(){return await fetch("/auth/v1/fed_cm/status",{method:"GET",headers:e.json})}async function d(){return await fetch("/auth/v1/password_policy",{method:"GET",headers:e.json})}async function f(){return await fetch("/auth/v1/pow",{method:"POST",headers:e.json})}async function y(t){return await fetch("/auth/v1/users/request_reset",{method:"POST",headers:a(),body:JSON.stringify(t)})}async function w(t){return await fetch("/auth/v1/providers/login",{method:"POST",headers:a(),body:JSON.stringify(t)})}async function m(){return await fetch("/auth/v1/events/test",{method:"POST",headers:a()})}async function p(t){return await fetch(`/auth/v1/users/${t}`,{method:"GET",headers:e.json})}async function v(t){return await fetch(`/auth/v1/users/${t}/webauthn`,{method:"GET",headers:a()})}async function T(t,n){return await fetch(`/auth/v1/users/${t}/webid/data`,{method:"PUT",headers:a(),body:JSON.stringify(n)})}async function j(t,n){return await fetch(`/auth/v1/users/${t}/webauthn/delete/${n}`,{method:"DELETE",headers:a()})}export{i as a,f as b,u as c,y as d,T as e,o as f,h as g,d as h,v as i,p as j,m as k,c as l,r as m,w as p,j as w};
