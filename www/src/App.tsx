import Status from "./components/status";
import './App.css';
import { useState } from "react";
import { setConstantValue } from "typescript";

const code = `
; COMMENT DESCRIPTION
            MVI A, 5dh
  NEXT:     DCR A
            JNZ NEXT 
            HLT
`

function App(props: any) {

  const mp = props.wasm.PP8085.new();

  const [cpu, setCpu] = useState(mp);
  const [source, setSource] = useState("");
  const [n, setN] = useState(false);

  const bin = props.wasm.parse_wasm(code);
  let rom = props.wasm.Memory.new_from_js(bin, 1024*8);
  cpu.load_memory(rom);
  
  const handleClick = () => {
    cpu.run_next();
    setN(!n);
  }

  const handleCompile = () => {
    cpu.reset();
    setN(!n);
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
