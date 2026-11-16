'use strict';
// A closed, in-memory TypeScript project. Learner text is parsed, never evaluated.
const fs = require('node:fs');
const path = require('node:path');
const tsPath = fs.realpathSync(process.argv[2]);
const ts = require(tsPath);
const lib = path.dirname(tsPath);
const request = JSON.parse(fs.readFileSync('request.json','utf8'));
const files = new Map([['/workspace/'+request.file,request.source],['/workspace/'+(request.file==='answer.ts'?'playground.ts':'answer.ts'),request.sibling]]);
function library(file) {
  if (!file.endsWith('.d.ts')) return undefined;
  try { const real=fs.realpathSync(file); if(real.startsWith(lib+path.sep))return fs.readFileSync(real,'utf8'); } catch {}
}
const host={
 getScriptFileNames:()=>[...files.keys()], getScriptVersion:()=> '1',
 getScriptSnapshot:file=> {const source=files.get(file)??library(file);return source===undefined?undefined:ts.ScriptSnapshot.fromString(source)},
 getCurrentDirectory:()=>'/workspace',getDefaultLibFileName:()=>path.join(lib,'lib.es2022.full.d.ts'),
 getCompilationSettings:()=>({strict:true,target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,types:[],noUncheckedIndexedAccess:true}),
 fileExists:file=>files.has(file)||library(file)!==undefined,readFile:file=>files.get(file)??library(file),
 readDirectory:()=>[],directoryExists:dir=>dir==='/workspace'||dir===lib,
 resolveModuleNames:names=>names.map(name=>name==='./answer'?{resolvedFileName:'/workspace/answer.ts',extension:ts.Extension.Ts}:name==='./playground'?{resolvedFileName:'/workspace/playground.ts',extension:ts.Extension.Ts}:undefined)
};
const service=ts.createLanguageService(host);
const chars=Array.from(request.source);
const before=chars.slice(0,request.cursor).join('');
const prefix=before.match(/[\p{ID_Continue}$]+$/u)?.[0]??'';
const suffix=chars.slice(request.cursor).join('').match(/^[\p{ID_Continue}$]+/u)?.[0]??'';
const file='/workspace/'+request.file;
const info=service.getCompletionsAtPosition(file,before.length,{includeCompletionsForModuleExports:false,includeCompletionsWithInsertText:false});
const entries=(info?.entries??[]).filter(x=>!x.source&&x.name!==prefix&&x.name.toLowerCase().startsWith(prefix.toLowerCase())).sort((a,b)=>a.sortText.localeCompare(b.sortText)||a.name.localeCompare(b.name)).slice(0,32).map(entry=>{
 let detail='';try { const d=service.getCompletionEntryDetails(file,before.length,entry.name,{},undefined,{});detail=ts.displayPartsToString(d?.displayParts).slice(0,350); }catch{}
 return {label:entry.name,kind:entry.kind,detail,insert:entry.name,start:request.cursor-Array.from(prefix).length,end:request.cursor+Array.from(suffix).length};
});
fs.writeFileSync('result.json',JSON.stringify(entries));
service.dispose();
