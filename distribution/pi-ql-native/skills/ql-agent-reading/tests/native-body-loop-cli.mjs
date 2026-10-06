import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { createHash } from "node:crypto";

// Actual native CLI/RPC body, its acting provider, owner tools and event hooks.
const [executable, body, cwdArg, eventFile, provider, model, outputFile, scenario="tool"] = process.argv.slice(2);
assert(executable && ["pi","prime"].includes(body) && cwdArg && eventFile && provider && model && outputFile);
assert(["tool","encounter"].includes(scenario));
const cwd = resolve(cwdArg); await mkdir(cwd,{recursive:true});
const event = JSON.parse(await readFile(eventFile,"utf8")); delete event.bindings;
event.observed = [
  {field:"lens",value:"L2'",origin:"observed",basis_refs:[event.material.ref]},
  {field:"local-position",value:3,origin:"observed",basis_refs:[event.material.ref]},
  {field:"coordinate-face",value:"direct",origin:"observed",basis_refs:[event.material.ref]},
  {field:"musical-basis",value:"chromatic",origin:"observed",basis_refs:[event.material.ref]},
];
const trace=[], pending=new Map(), turns=[];
let buffer="", bytes=0, stderr="", sequence=0, closed=false;
const child=spawn(executable,["--mode","rpc","--offline","--no-session","--no-context-files",
  "--no-themes","--no-prompt-templates","--provider",provider,"--model",model,"--thinking","low",
  "--tools",scenario==="tool"?"ql_project_event":body==="prime"?"ipython":"read"],{cwd,shell:false,detached:true,stdio:["pipe","pipe","pipe"]});
const fail=error=>{for(const p of pending.values())p.reject(error);pending.clear();for(const t of turns)t.reject(error);turns.length=0;};
const receive=row=>{
  trace.push(row);
  if(row.type==="response" && pending.has(row.id)){
    const p=pending.get(row.id);pending.delete(row.id);row.success===false?p.reject(new Error(JSON.stringify(row))):p.resolve(row);
  }
  if(row.type==="agent_end" && turns.length)turns.shift().resolve(row);
};
child.stdout.on("data",chunk=>{
  bytes+=chunk.length;if(bytes>8*1024*1024){fail(new Error("native RPC trace exceeds bound"));return;}
  buffer+=chunk.toString();let at;
  while((at=buffer.indexOf("\n"))>=0){const line=buffer.slice(0,at);buffer=buffer.slice(at+1);
    if(line.trim())try{receive(JSON.parse(line));}catch(error){fail(error);}}
});
child.stderr.on("data",chunk=>{bytes+=chunk.length;stderr+=chunk.toString();if(bytes>8*1024*1024)fail(new Error("native diagnostics exceed bound"));});
child.on("error",fail);child.on("close",code=>{closed=true;fail(new Error(`native CLI closed ${code}: ${stderr.slice(-1000)}`));});
const request=(type,fields={})=>new Promise((resolvePromise,reject)=>{
  const id=`ql-body-${++sequence}`;pending.set(id,{resolve:resolvePromise,reject});
  child.stdin.write(JSON.stringify({id,type,...fields})+"\n");
});
const turn=async text=>{
  const ended=new Promise((resolvePromise,reject)=>turns.push({resolve:resolvePromise,reject}));
  await request("prompt",{message:text});return ended;
};
const timer=setTimeout(()=>{fail(new Error("native body loop exceeded 180 seconds"));try{process.kill(-child.pid,"SIGTERM");}catch{}},180000);
let failure;
try{
  const commands=await request("get_commands");assert(JSON.stringify(commands).includes('ql-mode'));
  await request("prompt",{message:"/ql-mode on"});
  const stateInput="QL state: "+JSON.stringify({lens:"L2'","local-position":3,
    "coordinate-face":"direct","musical-basis":"chromatic"});
  const expectedRef=scenario==="tool"?event.event_ref:
    "ql:event:sha256:"+createHash("sha256").update(stateInput).digest("hex");
  await turn(scenario==="tool"?
    "Call ql_project_event once with this exact request, preserve its native result and return one sentence. "
      +"Do not calculate an alternative reading.\n"+JSON.stringify({event,requested_heads:["lens"]}):stateInput);
  const result=trace.find(row=>row.type==="tool_execution_end"&&row.toolName==="ql_project_event"&&!row.isError);
  if(scenario==="tool") assert(result,"real native QL tool must execute successfully");
  else assert(!result,"input-hook proof must not rely on an acting QL tool call");
  const continued = await turn("Continue from the native QL receipt. State its event reference and harmonic pitch class in one sentence. Do not call tools.");
  const reply = continued.messages.filter(m=>m.role==="assistant").flatMap(m=>m.content)
    .filter(c=>c.type==="text").map(c=>c.text).join("\n");
  const messages=(await request("get_messages")).data.messages;
  const readings=messages.filter(m=>m.customType==="ql-native-event-context");
  assert(readings.length>0,"actual next-turn context must contain the owner receipt");
  assert.equal(readings.at(-1).details.body,body);
  const reading=readings.at(-1);
  const native=result?.result.details.native_receipt ?? JSON.parse(reading.content.slice(reading.content.indexOf("\n")+1));
  assert.equal(native.event.event_ref,expectedRef);
  assert.deepEqual(native.decision_head_ids,[]);
  const pitch=native.harmonic.harmonic.find(f=>f.field==="pitch-class").value;
  assert.equal(pitch,11);
  assert(reply.includes(expectedRef),"the acting agent must use the exact retained event reference");
  assert(new RegExp(`\\b${pitch}\\b`).test(reply),"the acting agent must use the native harmonic pitch");
  const count=readings.length;
  await request("prompt",{message:"/ql-mode off"});
  await turn("Reply with the single word continued. Do not call tools.");
  const after=(await request("get_messages")).data.messages;
  assert.equal(after.filter(m=>m.customType==="ql-native-event-context").length,count);
  assert(!after.some(m=>m.role==="assistant"&&m.stopReason==="error"));
  process.stdout.write(JSON.stringify({body,scenario,native_tool_success:!!result,continuation_readings:count,
    actual_reply_event_ref:expectedRef,actual_reply_pitch_class:pitch,
    decision_provider_calls:0,mode_off_ordinary_continuation:true})+"\n");
}catch(error){failure=String(error);throw error;}
finally{
  clearTimeout(timer);
  await writeFile(outputFile,JSON.stringify({body,scenario,provider,model,failure:failure??null,trace,stderr},null,2)+"\n");
  if(!closed){const exited=new Promise(resolvePromise=>child.once("close",resolvePromise));
    try{process.kill(-child.pid,"SIGTERM");}catch{}
    const kill=setTimeout(()=>{try{process.kill(-child.pid,"SIGKILL");}catch{}},2500);
    await exited;clearTimeout(kill);}
}
