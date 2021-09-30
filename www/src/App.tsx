import Header from "./components/header";
import WarnDialog from "./components/dialog";
import MemTable from "./components/memory";
import Status from "./components/status";
import Footer from "./components/footer";
import React from 'react';
import './App.css'
import AceEditor from 'react-ace'
import "ace-builds/src-noconflict/theme-github"
import "ace-builds/src-noconflict/theme-dracula"
import "ace-builds/src-noconflict/mode-assembly_x86"
// MUI
import { Typography, IconButton, Slider, ButtonGroup, Button, CssBaseline, Box, Divider } from '@mui/material';
import {ThemeProvider} from "@emotion/react";
import { createTheme } from '@mui/material/styles';
// wasm
import { PP8085 } from "pp8085";
import IoPorts from "./components/ioports";

const code = `; Count down from 15 to 0
            MVI A, fh
  NEXT:     DCR A
            JNZ NEXT 
            HLT`

interface wasm_state {
    cpu: PP8085,
    source: string,
    parse_code: (data:string)=>Uint8Array,
    loading: boolean,
    warn_open: boolean,
    warning: string,
    dark: boolean,
}

const fonts = [
    '-apple-system',
    'BlinkMacSystemFont',
    '"Segoe UI"',
    'Roboto',
    '"Helvetica Neue"',
    'Arial',
    'sans-serif',
    '"Apple Color Emoji"',
    '"Segoe UI Emoji"',
    '"Segoe UI Symbol"',
  ].join(',')

const lightTheme = createTheme({
  palette: {
    mode: "light"
  },
  typography: {
    fontFamily: fonts
  }
});

const darkTheme = createTheme({
  palette: {
    mode: "dark"
  },
  typography: {
    fontFamily: fonts 
  }
});

const mem_size = 1024*8;
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
    this.handlePause = this.handlePause.bind(this);
    this.handleCompile = this.handleCompile.bind(this);
    this.handleStep = this.handleStep.bind(this);
    this.handleReset = this.handleReset.bind(this);
    this.handleSpeed = this.handleSpeed.bind(this);
    this.handleWarnClose = this.handleWarnClose.bind(this);
    this.handleTheme = this.handleTheme.bind(this);
    this.handleIoEdit = this.handleIoEdit.bind(this);
    this.handleIOAdd = this.handleIOAdd.bind(this);
    this.handleIORemove = this.handleIORemove.bind(this);
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
      source: code,
      parse_code: wasm.parse_wasm,
      loading: false,
      dark: false,
      warn_open: false,
    });
  }

  handleChange(e: string) {
    this.setState(state => {
      return {
        ...state,
        source: e,
      };
    });
  }
  
  handleCompile() {
    let bin: Uint8Array;
    try {
      bin = this.state.parse_code(this.state.source);
    } catch(err) {
      this.setState(state=> {
        return {
          ...state,
          warn_open: true,
          warning: err as string
        }
      })
      return;
    }

    const rom = wasm.Memory.new_from_js(bin, mem_size);
    this.state.cpu.load_memory(rom);
    this.state.cpu.reset();
    if (this.run_interval != null) {
      clearInterval(this.run_interval);
      this.run_interval = null;
    }
    this.setState(state => state);
  }

  handleRun() {
    if (this.run_interval == null) {
      this.run_interval = setInterval(()=> {
        this.handleStep();
        if (this.state.cpu.get_hlt() && this.run_interval != null) {
          clearInterval(this.run_interval);
          this.run_interval = null;
          this.setState(state => state);
        }
      }, this.run_speed);
      this.setState(state => state);
    }
  }

  handlePause() {
    if (this.run_interval) {
      clearInterval(this.run_interval);
      this.run_interval = null;
      this.setState(state => state);
    }
  }

  handleStep() {
    this.state.cpu.run_next();
    this.setState(state => state);
  }

  handleReset() {
    if (this.run_interval != null) {
      clearInterval(this.run_interval);
      this.run_interval = null;
    }
    this.state.cpu.reset();
    this.setState(state => state);
  }

  handleSpeed(_e: Event, value: number | Array<number>, _activeThumb: number) {
      this.run_speed = 2000 - (value as number);
  }

  handleWarnClose() {
    this.setState(state=> {
      return {
        ...state,
        warn_open: false,
      }
    })
  }

  handleTheme() {
    this.setState(state => {
      return {
        dark: !state.dark,
      }
    })
  }

  handleIoEdit(addr: number, data: number) {
    this.state.cpu.write_io(addr, data);
    this.setState(state=>state);
  }

  handleIOAdd(addr:number) {
    if (addr <= 0xff) {
      this.state.cpu.add_io_port(addr);
      this.setState(state=>state);
    }
  }

  handleIORemove(addr:number) {
    this.state.cpu.remove_io_port(addr);
    this.setState(state=>state);
  }

  render () {
    if (this.state != null) {
      return (
        <div className="App">
          <ThemeProvider theme={this.state.dark ? darkTheme : lightTheme}>
          <CssBaseline>
            
          <Box display="flex" justifyContent="center" alignItems="center">
            <Box display="flex" flexDirection="column" alignItems="center" order={1} p={1} m={2} alignSelf="flex-start">
              <Header dark={this.state.dark}/>

              <AceEditor onChange={this.handleChange} mode="assembly_x86" defaultValue={code} theme={this.state.dark?"dracula":"github"} style={{resize: 'none'}}/>

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
                      <Button color="secondary" onClick={this.handlePause} disabled={this.state.cpu.get_hlt() || this.run_interval == null}>Pause</Button>
                    </ButtonGroup>
                  </Box>
              </Box>

              <Box display="flex" alignItems="center" sx={{width: "100%"}}>
                <IconButton onClick={this.handleTheme}>
                  <img
                    style={this.state.dark ? { filter: 'invert(1)' } : undefined}
                    src='data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB4AAAAeCAYAAAA7MK6iAAAABmJLR0QA/wD/AP+gvaeTAAABkklEQVRIie3WvUodQRQH8J969YIiBlIJPoEPoA8gWAliYgLiLbXyo/Gjip0vYGGRh0hhSBVI0kUuhiRdqkQtRFBQxMpbpdgRr+Dunf0AQfzDYWd35/z/M2fOmRmeEY+JYIVQKyH8Njy/FHHuLiFcClnCPRXwp3KkCddwjkYGaStYGho4U2A5G7jJEB8KVsQ3Wnw2h8+bsqLtRPOhXcMivknCeIavWHAX0vmYgXblGMAwPuIldnEQvo9hSZIT0zjNwdkRdfzEHvof+N8fBvUDfVUKr+Bfimi7+JFk9pWhibWIfhvYjyFsr7FJvG57b+EdrjDqbk2zcICt0B7Ctvuh/4DP5Nsy8yRiZWhiPaLfpshQx2IVh7KTawDHWK5SuI7fkpJ5SHwAn/BLZDnlWbeRQP5CsoE0g/+4pIQuMIWTHJyZeIW50K5Lanof18G+S8J7O9M5zJQVLXJIzKroZEojGAxWxDcVvbjs4LgTLA2NwNGbV7xTdr4PVogja+fKutbEIpXj0W6ZZW6SLfzB34rG8sTxHxQCSoItZf48AAAAAElFTkSuQmCC'
                    alt='change theme icon'
                  />
                </IconButton>
                <Divider orientation="vertical" flexItem variant="middle"/>
                <Typography m={3}>Emulation Speed</Typography>
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

            <Box order={2} p={1} m={6} alignSelf="flex-start" display="flex" flexDirection="column" sx={{minWidth: 500, maxWidth: 600}}>
              <Box m={1}>
                <Status cpu={this.state.cpu}/>
              </Box>
              <Box m={1}>
                <MemTable ptr={this.state.cpu.get_memory_ptr()} memory={memory.memory} size={mem_size}/>
              </Box>
              <Box m={1}>
                <IoPorts handleEdit={this.handleIoEdit} handleAdd={this.handleIOAdd} handleRemove={this.handleIORemove} ports={this.state.cpu.get_io_ports()}/>
              </Box>
            </Box>
          </Box>

          <Footer dark={this.state.dark}/>

          <WarnDialog handleClose={this.handleWarnClose} open={this.state.warn_open} heading="Opps Did A BOO.. BOO.." description={this.state.warning}/>

          </CssBaseline>
          </ThemeProvider>
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
