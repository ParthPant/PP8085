import {useState} from 'react'
import {Table, TableBody, TableCell, TableRow, TableContainer, Paper, TableHead, TablePagination, TableFooter} from '@mui/material'

function MemTable(props: {memory: any, ptr: number, size: number}) {
    console.log(props.memory.buffer);
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

    return (
        <Paper elevation={3} sx={{p:2}}>
            <h2>Memory</h2>
            <p>Showing larger number of rows slows down emulation</p>
            <TableContainer sx={{ minHeight:300, maxHeight: 300, minWidth: 500 }}>
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
                            count={100}
                            page={page}
                            onPageChange={handleChangePage}
                            rowsPerPage={rowsPerPage}
                            onRowsPerPageChange={handleChangeRowsPerPage}
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