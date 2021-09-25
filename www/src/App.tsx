import Status from "./components/status";
import './App.css';
import { useState } from "react";
import { setConstantValue } from "typescript";

function App(props: any) {

  const mp = props.wasm.PP8085.new();
  const rom = props.wasm.Memory.new(10324*8);
  mp.load_memory(rom);

  const code = `
  ; COMMENT DESCRIPTION
              MVI A, 5dh
    NEXT:     DCR A
              JNZ NEXT 
              HLT
  `
  const [cpu, setCpu] = useState(mp);
  const [source, setSource] = useState("");
  const [n, setN] = useState(0);
  
  const handleClick = () => {
    cpu.run_next();
    setN(n+1);
  }

  const handleCompile = () => {
    const bin = props.wasm.parse_wasm(source);
    const rom = props.wasm.Memory.new_from_js(bin, 10324*8);
    cpu.reset();
    cpu.load_memory(rom);
    setN(n+1);
  }

  const handleChange = (e: any) => {
    setSource(e.target.value);
  }

  return (
    <div className="App">
      <Status cpu={cpu}/>
      <textarea onChange={handleChange} defaultValue={code}></textarea>
      <button onClick={handleClick}>Next</button>
      <button onClick={handleCompile}>Compile</button>
    </div>
  );
}

export default App;
