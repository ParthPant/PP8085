import { Table, TableRow, TableCell, TableHead, Paper} from '@mui/material'

import { PP8085 } from "pp8085";

function Status(props: {cpu: PP8085}) {
    if (props.cpu) {
        return (
            <Paper elevation={3}>
                <Table>
                    <TableHead>
                        <TableCell>Register</TableCell>
                        <TableCell>Contents</TableCell>
                        <TableCell>Register</TableCell>
                        <TableCell>Contents</TableCell>
                    </TableHead>
                    <TableRow>
                        <TableCell>A</TableCell>
                        <TableCell>0x{props.cpu.get_a().toString(16)}</TableCell>
                        <TableCell>F</TableCell>
                        <TableCell>0b{props.cpu.get_f().toString(2)}</TableCell>
                    </TableRow>
                    <TableRow>
                        <TableCell>B</TableCell>
                        <TableCell>0x{props.cpu.get_b().toString(16)}</TableCell>
                        <TableCell>C</TableCell>
                        <TableCell>0x{props.cpu.get_c().toString(16)}</TableCell>
                    </TableRow>
                    <TableRow>
                        <TableCell>D</TableCell>
                        <TableCell>0x{props.cpu.get_d().toString(16)}</TableCell>
                        <TableCell>E</TableCell>
                        <TableCell>0x{props.cpu.get_e().toString(16)}</TableCell>
                    </TableRow>
                    <TableRow>
                        <TableCell>H</TableCell>
                        <TableCell>0x{props.cpu.get_h().toString(16)}</TableCell>
                        <TableCell>L</TableCell>
                        <TableCell>0x{props.cpu.get_l().toString(16)}</TableCell>
                    </TableRow>
                    <TableRow>
                        <TableCell>SP</TableCell>
                        <TableCell>0x{props.cpu.get_sp().toString(16)}</TableCell>
                        <TableCell>PC</TableCell>
                        <TableCell>0x{props.cpu.get_pc().toString(16)}</TableCell>
                    </TableRow>
                    <TableRow>
                        <TableCell>IR</TableCell>
                        <TableCell>0x{props.cpu.get_ir().toString(16)}</TableCell>
                    </TableRow>
                </Table>
            </Paper>
        )
    } else {
        return (
            <p>loading</p>
        )
    }
}

export default Status;