import {useState} from 'react'
import {Typography, Table, TableBody, TableCell, TableRow, TableContainer, Paper, TableHead, TablePagination, TableFooter} from '@mui/material'

function MemTable(props: {memory: any, ptr: number, size: number}) {
    const d = new Uint8Array(props.memory.buffer, props.ptr, props.size);
    const data = Array.from(d);

    const [page, setPage] = useState<number>(0);
    const [rowsPerPage, setRowsPerPage] = useState<number>(10);

    const handleChangePage = (
        event: React.MouseEvent<HTMLButtonElement> | null,
        newPage: number,
    ) => {
        setPage(newPage);
    };

    const handleChangeRowsPerPage = (
        event: React.ChangeEvent<HTMLInputElement>,
    ) => {
        setRowsPerPage(+event.target.value);
        setPage(0);
    };

    const defaultLabelDisplayedRows = ({ from, to, count }: {from: number, to:number, count: number}) => { 
        return `${from}-${to} of ${count != -1 ? count : `${props.size}`}`;
    }

    return (
        <Paper elevation={3} sx={{p:2}}>
            <Typography variant="h4" align="left">Memory</Typography>
            <Typography variant="subtitle2" align="left">Showing larger number of rows slows down emulation</Typography>
            <Typography variant="subtitle2" align="left" color="primary" paragraph>Memory is limited to {props.size} bits, accessing outside the range will cause undefined behaviour.</Typography>
            <TableContainer sx={{ minHeight:300, maxHeight: 300}}>
                <Table padding="checkbox" stickyHeader size="small">
                    <TableHead>
                        <TableRow>
                            <TableCell>Address</TableCell>
                            <TableCell>Byte</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {data.slice(page*rowsPerPage, page*rowsPerPage+rowsPerPage).map((n, i:number) => (
                            <TableRow>
                                <TableCell>{(page*rowsPerPage+i).toString(16).toUpperCase()}</TableCell>
                                <TableCell>{n.toString(16).toUpperCase()}</TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                    <TableFooter>
                        <TableRow>
                            <TablePagination
                            component="div"
                            count={-1}
                            page={page}
                            onPageChange={handleChangePage}
                            rowsPerPage={rowsPerPage}
                            onRowsPerPageChange={handleChangeRowsPerPage}
                            labelDisplayedRows={defaultLabelDisplayedRows}
                            rowsPerPageOptions={[10, 50, 100, 256]}
                            />
                        </TableRow>
                    </TableFooter>
                </Table>
            </TableContainer>
        </Paper>
    )
}

export default MemTable;