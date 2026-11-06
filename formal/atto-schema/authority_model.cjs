"use strict";
const KEYS=["typespec","authoredJson","desired","diesel","seaorm","shadow"];
const UNSET=0,MATCH=1,DRIFT=2;
const initial={lane:[0,0,0,0,0,0],approved:false,blocked:false,published:false,mutationDetected:false,driftDetected:false,allMatched:false};
function must(ok,why){if(!ok)throw Error("DPM authority model: "+why);}
function converged(s){return s.lane.every(x=>x===MATCH)&&!s.blocked;}
function valid(s){
 must(s.lane.length===6 && s.lane.every(x=>[UNSET,MATCH,DRIFT].includes(x)),"invalid peer authority evidence");
 must(!s.published || (s.approved && converged(s)),"published without every independently checked participant");
 must(!s.approved || converged(s),"approval without six-way semantic agreement");
 must(!s.blocked || !s.approved,"approval survived discrepancy");
}
function next(s){
 const out=[];
 for(let i=0;i<6;i++){
  if(s.lane[i]===UNSET){
   for(const value of [MATCH,DRIFT]){
    const lane=s.lane.map((x,j)=>i===j?value:x);
    out.push(["check-"+KEYS[i]+"-"+(value===MATCH?"match":"drift"),
     {...s,lane,blocked:s.blocked||value===DRIFT,driftDetected:s.driftDetected||value===DRIFT}]);
   }
  }
 }
 if(converged(s)&&!s.approved)out.push(["independent-review-approve",{...s,approved:true,allMatched:true}]);
 if(s.approved&&!s.published)out.push(["publish-immutable-artifact",{...s,published:true}]);
 if(s.blocked&&!s.mutationDetected)out.push(["deny-forced-fallback",{...s,mutationDetected:true}]);
 return out;
}
function explore(){
 const limit=9,seen=new Set([JSON.stringify(initial)]),queue=[[initial,0]];
 let edges=0;
 for(let head=0;head<queue.length;head++){
  const [s,depth]=queue[head];valid(s);if(depth===limit)continue;
  for(const [name,t] of next(s)){
   valid(t);edges++;const key=JSON.stringify(t);
   if(!seen.has(key)){seen.add(key);queue.push([t,depth+1]);}
  }
 }
 const states=[...seen].map(JSON.parse);
 for(const prop of ["approved","published","blocked","mutationDetected","driftDetected","allMatched"])
  must(states.some(s=>s[prop]),"missing positive/negative witness: "+prop);
 for(const bad of [
  {...initial,published:true},{...initial,approved:true},
  {...initial,blocked:true,approved:true},{...initial,lane:[3,0,0,0,0,0]}
 ]){let caught=false;try{valid(bad);}catch{caught=true;}must(caught,"invariant mutation accepted");}
 return {states:seen.size,transitions:edges,participants:6,bound:limit,claim:"finite source-level peer-authority model only"};
}
const result=explore();
if(typeof console!=="undefined")console.log(JSON.stringify(result));
