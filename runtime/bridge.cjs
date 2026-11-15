'use strict';
// Trusted local bridge. Learner JavaScript is evaluated only in the separate VM context.
const fs = require('node:fs');
const vm = require('node:vm');
const { Worker, isMainThread, parentPort, workerData } = require('node:worker_threads');
const path = require('node:path');
const limit = (value, max = 2000) => String(value).slice(0, max);
const emit = event => {
  if (event.type === 'paused') {
    event.frames = event.frames.slice(0, 8).map(frame => ({ ...frame,
      variables: frame.variables.slice(0, 16).map(variable => ({ ...variable,
        name: limit(variable.name, 80), value: limit(variable.value, 200) })) }));
    while (Buffer.byteLength(JSON.stringify(event)) > 60000 && event.frames.length > 1) event.frames.pop();
  }
  fs.writeSync(1, JSON.stringify(event) + '\n');
};

if (isMainThread) {
  const config = JSON.parse(fs.readFileSync('session.json', 'utf8'));
  const worker = new Worker(__filename, { workerData: config });
  worker.on('error', error => { emit({ type: 'failure', text: limit(error.stack) }); process.exitCode = 1; });
  worker.once('message', () => {
    try {
      const context = vm.createContext(Object.create(null), {
        name: 'TypeDojo workspace', codeGeneration: { strings: false, wasm: false },
        microtaskMode: 'afterEvaluate',
      });
      vm.runInContext(`globalThis.__modules = {answer: {}, playground: {}};
        globalThis.__require = name => {
          if (name === './answer') return __modules.answer;
          throw new Error('Only ./answer can be imported in this workspace.');
        };`, context);
      for (const file of ['answer', 'playground']) {
        const source = fs.readFileSync(path.join('out', file + '.js'), 'utf8');
        context.__entry = new vm.Script('(function(exports, require){\n' + source + '\n})', {
          filename: 'typedojo:///' + file + '.js',
        }).runInContext(context);
        vm.runInContext(`__entry(__modules.${file}, __require)`, context,
          config.debug ? {} : { timeout: 4500 });
      }
      worker.postMessage({ success: true });
    } catch (error) {
      worker.postMessage({ success: false, error: limit(error.stack || error) });
    }
  });
} else {
  const inspector = require('node:inspector');
  const readline = require('node:readline');
  const session = new inspector.Session();
  session.connectToMainThread();
  const post = (method, params = {}) => new Promise((resolve, reject) =>
    session.post(method, params, (error, result) => error ? reject(error) : resolve(result)));
  let logCount = 0;
  let paused = [];
  const contexts = new Set();
  const scripts = new Map();
  const maps = new Map();
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  for (const file of ['answer', 'playground']) {
    const payload = JSON.parse(fs.readFileSync('out/' + file + '.js.map', 'utf8'));
    let originalLine = 0, originalColumn = 0, source = 0;
    const entries = [];
    payload.mappings.split(';').forEach((row, generatedLine) => {
      let column = 0;
      for (const segment of row.split(',')) {
        if (!segment) continue;
        const numbers = [];
        let value = 0, shift = 0;
        for (const char of segment) {
          const digit = alphabet.indexOf(char);
          if (digit < 0 || shift > 30) throw new Error('Invalid source map');
          value += (digit & 31) * 2 ** shift;
          if (digit & 32) { shift += 5; continue; }
          numbers.push(value & 1 ? -(value >> 1) : value >> 1);
          value = 0; shift = 0;
        }
        column += numbers[0] || 0;
        if (numbers.length >= 4) {
          source += numbers[1]; originalLine += numbers[2]; originalColumn += numbers[3];
          if (source === 0) entries.push({ generatedLine: generatedLine + 1, column,
            line: originalLine + 1, originalColumn: originalColumn + 1 });
        }
      }
    });
    maps.set('typedojo:///' + file + '.js', { file: file + '.ts', entries, original: fs.readFileSync(file + '.ts', 'utf8') });
  }
  function sourceLocation(url, lineNumber, columnNumber) {
    const map = maps.get(url);
    if (!map) return null;
    const entries = map.entries.filter(e => e.generatedLine === lineNumber && e.column <= columnNumber);
    const entry = entries.at(-1);
    return entry ? { file: map.file, line: entry.line, column: entry.originalColumn } : null;
  }
  function location(loc) {
    return sourceLocation(scripts.get(loc.scriptId), loc.lineNumber, loc.columnNumber);
  }
  function mappedStack(text) {
    return text.replace(/typedojo:\/\/\/(answer|playground)\.js:(\d+):(\d+)/g, (match, file, line, column) => {
      const mapped = sourceLocation('typedojo:///' + file + '.js', Number(line) - 1, Number(column) - 1);
      return mapped ? mapped.file + ':' + mapped.line + ':' + mapped.column : match;
    });
  }
  function display(remote) {
    if (!remote) return '<unavailable>';
    if ('value' in remote) return limit(typeof remote.value === 'string' ? JSON.stringify(remote.value) : JSON.stringify(remote.value));
    if (remote.unserializableValue) return remote.unserializableValue;
    if (remote.preview?.properties?.length) {
      return limit((remote.description || remote.type) + ' { ' + remote.preview.properties.slice(0, 12)
        .map(p => p.name + ': ' + (p.value ?? p.type)).join(', ') + (remote.preview.overflow ? ', …' : '') + ' }');
    }
    return limit(remote.description || remote.type);
  }
  async function frameData(frame) {
    const variables = [];
    for (const scope of frame.scopeChain.filter(s => ['local', 'closure', 'block', 'catch', 'module'].includes(s.type)).slice(0, 4)) {
      if (!scope.object.objectId) continue;
      const properties = await post('Runtime.getProperties', {
        objectId: scope.object.objectId, ownProperties: true, generatePreview: true,
      });
      for (const property of (properties.result || []).slice(0, 30)) {
        if (['exports', 'require'].includes(property.name)) continue;
        const original = maps.get(scripts.get(frame.location.scriptId))?.original || '';
        if (/_\d+$/.test(property.name) && !original.includes(property.name)) continue;
        variables.push({ name: limit(property.name, 120), value: property.value ? display(property.value) : property.get ? '<getter>' : '<uninitialized>', scope: scope.type });
      }
    }
    return { id: frame.callFrameId, name: limit(frame.functionName || '(module)', 120),
      ...location(frame.location), variables };
  }
  async function publishPause(params) {
    paused = params.callFrames.filter(frame => location(frame.location)).slice(0, 20);
    if (!paused.length) { await post('Debugger.resume'); return; }
    const frames = [];
    for (const frame of paused) frames.push(await frameData(frame));
    emit({ type: 'paused', reason: params.reason === 'exception' ? 'exception' : params.hitBreakpoints?.length ? 'breakpoint' : params.reason === 'step' ? 'step' : 'pause', frames });
  }
  session.on('Runtime.executionContextCreated', ({ params }) => {
    if (params.context.name === 'TypeDojo workspace') contexts.add(params.context.id);
  });
  session.on('Debugger.scriptParsed', ({ params }) => scripts.set(params.scriptId, params.url));
  session.on('Runtime.consoleAPICalled', ({ params }) => {
    if (!contexts.has(params.executionContextId)) return;
    if (++logCount <= 500) emit({ type: 'log', level: params.type,
      text: limit(params.args.map(arg => arg.type === 'string' ? limit(arg.value) : display(arg)).join(' '), 4000) });
    else if (logCount === 501) emit({ type: 'log', level: 'warning', text: 'Console limit reached (500 messages). Further logs are omitted.' });
  });
  session.on('Debugger.paused', ({ params }) => {
    // Mark paused immediately so the host execution deadline does not count inspection time.
    emit({ type: 'pausing' });
    publishPause(params).catch(error => emit({ type: 'failure', text: limit(error.message) }));
  });
  session.on('Debugger.resumed', () => { paused = []; emit({ type: 'running' }); });
  parentPort.on('message', result => {
    if (!result.success) emit({ type: 'log', level: 'error', text: mappedStack(result.error) });
    emit({ type: 'finished', success: result.success });
    session.disconnect();
    process.exit(0);
  });
  const input = readline.createInterface({ input: fs.createReadStream(null, { fd: 0, autoClose: false }), crlfDelay: Infinity });
  input.on('line', async line => {
    try {
      const command = JSON.parse(line);
      const methods = { continue: 'Debugger.resume', pause: 'Debugger.pause',
        over: 'Debugger.stepOver', into: 'Debugger.stepInto', out: 'Debugger.stepOut' };
      if (methods[command.action]) await post(methods[command.action]);
      else if (command.action === 'evaluate') {
        const frame = paused.find(f => f.callFrameId === command.frame);
        if (!frame) throw new Error('Pause and select a stack frame before evaluating.');
        const result = await post('Debugger.evaluateOnCallFrame', { callFrameId: frame.callFrameId,
          expression: limit(command.expression, 1000), generatePreview: true, throwOnSideEffect: true });
        emit({ type: 'evaluation', text: result.exceptionDetails ? display(result.exceptionDetails.exception) : display(result.result) });
      }
    } catch (error) { emit({ type: 'commandError', text: limit(error.message) }); }
  });
  (async () => {
    await post('Runtime.enable');
    await post('Debugger.enable');
    await post('Debugger.setBlackboxPatterns', { patterns: ['^(?!typedojo:///).*'] });
    await post('Debugger.setPauseOnExceptions', { state: workerData.debug ? 'all' : 'none' });
    if (!workerData.debug) await post('Debugger.setSkipAllPauses', { skip: true });
    if (workerData.debug) {
      const bound = new Set();
      async function bind(url, entry) {
        const key = url + ':' + entry.generatedLine + ':' + entry.column;
        if (bound.has(key)) return;
        bound.add(key);
        await post('Debugger.setBreakpointByUrl', { url, lineNumber: entry.generatedLine, columnNumber: entry.column });
      }
      const requested = workerData.breakpoints || {};
      // Pause at the first executable playground statement even without user breakpoints.
      const first = maps.get('typedojo:///playground.js').entries[0];
      if (first) {
        await bind('typedojo:///playground.js', first);
      }
      for (const [file, lines] of Object.entries(requested)) {
        const url = 'typedojo:///' + file.replace(/\.ts$/, '.js');
        const map = maps.get(url);
        if (!map) continue;
        for (const line of lines) {
          const entry = map.entries.filter(e => e.line >= line).sort((a, b) => a.line - b.line || a.generatedLine - b.generatedLine || a.column - b.column)[0];
          if (!entry) { emit({ type: 'log', level: 'warning', text: file + ':' + line + ' has no executable code.' }); continue; }
          await bind(url, entry);
          emit({ type: 'breakpoint', file, requested: line, line: entry.line });
        }
      }
    }
    emit({ type: 'running' });
    parentPort.postMessage('ready');
  })().catch(error => { emit({ type: 'failure', text: limit(error.stack) }); process.exit(1); });
}
