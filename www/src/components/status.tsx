import { Typography, Table, TableRow, TableContainer, TableCell, TableHead, Paper, TableBody} from '@mui/material'
import { PP8085 } from "pp8085";

function Status(props: {cpu: PP8085}) {
    if (props.cpu) {
        return (
            <Paper elevation={3} sx={{p: 2}}>
            <Typography variant="h4" align="left">Registers</Typography>
                <TableContainer>
                    <Table size="small">
                        <TableHead>
                            <TableRow>
                                <TableCell>Register</TableCell>
                                <TableCell>Contents</TableCell>
                                <TableCell>Register</TableCell>
                                <TableCell>Contents</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            <TableRow>
                                <TableCell>A</TableCell>
                                <TableCell>0x{props.cpu.get_a().toString(16).toUpperCase()}</TableCell>
                                <TableCell>F</TableCell>
                                <TableCell>0b{props.cpu.get_f().toString(2).toUpperCase()}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>B</TableCell>
                                <TableCell>0x{props.cpu.get_b().toString(16).toUpperCase()}</TableCell>
                                <TableCell>C</TableCell>
                                <TableCell>0x{props.cpu.get_c().toString(16).toUpperCase()}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>D</TableCell>
                                <TableCell>0x{props.cpu.get_d().toString(16).toUpperCase()}</TableCell>
                                <TableCell>E</TableCell>
                                <TableCell>0x{props.cpu.get_e().toString(16).toUpperCase()}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>H</TableCell>
                                <TableCell>0x{props.cpu.get_h().toString(16).toUpperCase()}</TableCell>
                                <TableCell>L</TableCell>
                                <TableCell>0x{props.cpu.get_l().toString(16).toUpperCase()}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>SP</TableCell>
                                <TableCell>0x{props.cpu.get_sp().toString(16).toUpperCase()}</TableCell>
                                <TableCell>PC</TableCell>
                                <TableCell>0x{props.cpu.get_pc().toString(16).toUpperCase()}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>IR</TableCell>
                                <TableCell>0x{props.cpu.get_ir().toString(16).toUpperCase()}</TableCell>
                            </TableRow>
                        </TableBody>
                    </Table>
                </TableContainer>
            </Paper>
        )
    } else {
        return (
            <p>loading</p>
        )
    }
}

export default Status;