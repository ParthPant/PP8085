import {Paper, Typography, Button, Box} from "@mui/material"
import { DataGrid, GridColDef, GridCellEditCommitParams, MuiEvent, GridCallbackDetails, MuiBaseEvent} from '@mui/x-data-grid';
import React, { useState } from "react";

import PortDialog from './portdialog'

interface port {
    addr: number,
    data: number,
}

function IoPorts (props: {ports: any, handleEdit: (addr: number, data:number) => void, handleAdd: (addr: number)=>void, handleRemove:(addr: number)=>void}) {
    const {ports, handleEdit, handleAdd, handleRemove} = props;
    let rows: {id: number, addr: string, data: string}[] = [];
    const [addopen, setAddOpen] = useState<boolean>(false);
    const [removeopen, setRemoveOpen] = useState<boolean>(false);

    Object.entries(ports).map(([_key, value])=>
      {
        rows.push({id: (value as port).addr, addr: '0x'+(value as port).addr.toString(16), data: '0x'+(value as port).data.toString(16)});
        return null;
      }
    );

    const columns: GridColDef[] = [{ field: 'addr', headerName: 'Address', width: 180, editable: false },
    { field: 'data', headerName: 'Data', width: 180, editable: true}];

    const handleChange = (params: GridCellEditCommitParams, _event: MuiEvent<MuiBaseEvent>, _details: GridCallbackDetails) => {
      handleEdit(params.id as number, params.value as number);
    }

    const handleAddClose = () => {setAddOpen(false)};
    const handleRemoveClose = () => {setRemoveOpen(false)};

    const handleAddCommit = (addr: number) => {handleAdd(addr)};
    const handleRemoveCommit = (addr: number) => {handleRemove(addr)};

    return (
      <Paper elevation={3} sx={{p:2}}>
        <Typography variant="h4" align="left">I/O Ports</Typography>
        <Typography variant="subtitle1" align="left" color="primary">8085 only support I/O ports from 0x00 to 0xff</Typography>
        <Typography variant="subtitle2" align="left" color="secondary">Data and Addresses are represented in HEX</Typography>
        <DataGrid
          autoHeight
          disableColumnMenu
          disableColumnSelector
          columns={columns}
          rows={rows}
          onCellEditCommit={handleChange}
          rowsPerPageOptions={[5, 10, 15]}
        />

        <Box display="flex" justifyContent="flex-end">
          <Button sx={{m:1}} onClick={()=>setAddOpen(true)}>Add Port</Button>
          <Button sx={{m:1}} onClick={()=>setRemoveOpen(true)}>Remove Port</Button>
        </Box>

        <PortDialog description="Add a new I/O port" btn="Add" open={addopen} handleClose={handleAddClose} handleCommit={handleAddCommit}/>
        <PortDialog description="Remove I/O port" btn="Remove" open={removeopen} handleClose={handleRemoveClose} handleCommit={handleRemoveCommit}/>
      </Paper>
    )
}

export default IoPorts
