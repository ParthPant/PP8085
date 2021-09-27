import MemTable from "./components/memory";
import Status from "./components/status";
import Footer from "./components/footer";
import React from 'react';
import './App.css'
import AceEditor from 'react-ace'
import "ace-builds/src-noconflict/theme-github"
import "ace-builds/src-noconflict/mode-assembly_x86"
// MUI
import { Slider, ButtonGroup, Button } from '@mui/material';
// wasm
import { Memory, PP8085 } from "pp8085";
import { Box } from "@mui/system";
import Header from "./components/header";

const code = `
; COMMENT DESCRIPTION
            MVI A, fh
  NEXT:     DCR A
            JNZ NEXT 
            HLT
`

interface wasm_state {
    cpu: PP8085,
    rom: Uint8Array,
    source: string,
    parse_code: (data:string)=>Uint8Array,
    loading: boolean,
    running: boolean,
}

const mem_size = 1025*8;
let wasm: typeof import("pp8085");
let memory: any;

class App extends React.Component<{}, wasm_state>{
  run_interval: NodeJS.Timer | null = null;
  run_speed: number = 500;

  constructor(props: {}) {
    super(props);
    this.setState({loading: true})

    this.handleChange = this.handleChange.bind(this);
    this.handleRun = this.handleRun.bind(this);
    this.handleStop = this.handleStop.bind(this);
    this.handleCompile = this.handleCompile.bind(this);
    this.handleStep = this.handleStep.bind(this);
    this.handleReset = this.handleReset.bind(this);
    this.handleSpeed = this.handleSpeed.bind(this);
  }

  async componentDidMount () {
    wasm = await import('pp8085');
    memory = await import('pp8085/pp8085_lib_bg.wasm')
    const cpu = wasm.PP8085.new();
    const bin = wasm.parse_wasm(code);
    const rom = wasm.Memory.new_from_js(bin, mem_size);
    cpu.load_memory(rom);

    this.setState({
      cpu: cpu,
      // rom: mem,
      source: code,
      parse_code: wasm.parse_wasm,
      loading: false,
    });
  }

  handleChange(e: string) {
    this.setState(state => {
      return {
        source: e,
        cpu: state.cpu,
        // rom: state.rom,
        parse_code: state.parse_code,
        loading: state.loading,
      };
    });
  }
  
  handleCompile() {
    const bin = this.state.parse_code(this.state.source);
    const rom = wasm.Memory.new_from_js(bin, mem_size);
    this.state.cpu.load_memory(rom);
    this.state.cpu.reset();
    if (this.run_interval != null) {
      clearInterval(this.run_interval);
      this.run_interval = null;
    }
    // const mem = this.state.cpu.get_memory_contents();
    this.setState(state => {
      return {
        source: state.source,
        // rom: mem,
        cpu: state.cpu,
        parse_code: state.parse_code,
        loading: false,
      };
    });
  }

  handleRun() {
    if (this.run_interval == null) {
      this.run_interval = setInterval(()=> {
        this.handleStep();
        if (this.state.cpu.get_hlt() && this.run_interval != null) {
          clearInterval(this.run_interval);
          this.run_interval = null;
          // const mem = this.state.cpu.get_memory_contents();
          this.setState(state=> {
            return {
              source: state.source,
              // rom: mem,
              cpu: state.cpu,
              parse_code: state.parse_code,
              loading: false,
              running: false,
            };
          });
        }
      }, this.run_speed);

      // const mem = this.state.cpu.get_memory_contents();
      this.setState(state => {
        return {
          source: state.source,
          // rom: mem,
          cpu: state.cpu,
          parse_code: state.parse_code,
          loading: false,
          running: true,
        };
      });
    }
  }

  handleStop() {
    if (this.run_interval) {
      clearInterval(this.run_interval);
      this.run_interval = null;
      // const mem = this.state.cpu.get_memory_contents();
      this.setState(state => {
        return {
          source: state.source,
          // rom: mem,
          cpu: state.cpu,
          parse_code: state.parse_code,
          loading: false,
          running: false,
        };
      });
    }
  }

  handleStep() {
    this.state.cpu.run_next();
    // const mem = this.state.cpu.get_memory_contents();
    this.setState((state) => {
      return {
        cpu: state.cpu,
        // rom: mem,
        source: state.source,
        parse_code: state.parse_code,
        loading: state.loading,
        running: state.running,
      } 
    })
  }

  handleReset() {
    if (this.run_interval != null) {
      clearInterval(this.run_interval);
      this.run_interval = null;
    }
    this.state.cpu.reset();
    this.setState((state) => {
      return {
          source: state.source,
          // rom: state.rom,
          cpu: state.cpu,
          parse_code: state.parse_code,
          loading: state.loading,
          running: false,
      };
    });
  }

  handleSpeed(e: Event, value: number | Array<number>, activeThumb: number) {
      this.run_speed = 2000 - (value as number);
  }

  render () {
    if (this.state != null) {
      return (
        <div className="App">
          <Box display="flex" justifyContent="center" alignItems="center">
            <Box display="flex" flexDirection="column" alignItems="center" order={1} p={1} m={2}>
              <Header/>

              <AceEditor onChange={this.handleChange} mode="assembly_x86" defaultValue={code} theme="github" style={{resize: 'none'}}/>

              <Box display="flex" justifyContent="center" alignItems="center" sx={{p:3, textAlign: "center"}}>
                  <Box m={2}>
                    <Button variant="contained" onClick={this.handleCompile}>Compile & Load</Button>
                  </Box>

                  <Box m={2}>
                    <Button variant="contained" color="warning" onClick={this.handleReset}>Reset</Button>
                  </Box>

                  <Box m={2}>
                    <Button variant="contained" color="primary" onClick={this.handleStep} disabled={this.run_interval != null || this.state.cpu.get_hlt()}>Step</Button>
                  </Box>

                  <Box m={2}>
                    <ButtonGroup variant="outlined">
                      <Button color="success" onClick={this.handleRun} disabled={this.state.cpu.get_hlt() || this.run_interval != null}>Run</Button>
                      <Button color="secondary" onClick={this.handleStop} disabled={this.state.cpu.get_hlt() || this.run_interval == null}>Pause</Button>
                    </ButtonGroup>
                  </Box>
              </Box>

              <Box display="flex" alignItems="center" sx={{width: 300}}>
                <Box m={3}>Emulation Speed</Box>
                <Slider
                    defaultValue={3000-this.run_speed}
                    min={200}
                    max={3000}
                    valueLabelDisplay="auto"
                    onChange={this.handleSpeed}
                    disabled={this.run_interval != null}
                />
              </Box>
            </Box>

            <Box order={2} p={1} m={6} alignSelf="flex-start" display="flex" flexDirection="column">
              <Box m={1}>
                <Status cpu={this.state.cpu}/>
              </Box>
              <Box m={1}>
                <MemTable ptr={this.state.cpu.get_memory_ptr()} memory={memory.memory} size={mem_size}/>
              </Box>
            </Box>
          </Box>

          <Footer/>
        </div>
      )
    } else {
      return (
        <p>loading..</p>
      )
    }
  }
}

export default App;
