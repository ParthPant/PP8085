import {useState} from 'react'
import {Button, TextField, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle} from '@mui/material';

export default function PortDialog(props: {description: string, open: boolean, handleClose: ()=>void, handleCommit: (addr: number)=>void, btn: string}) {
  const {description, open, handleClose, handleCommit, btn} = props;

  const [addr, setAddr] = useState<number>(0);

  return (
    <div>
      <Dialog open={open} onClose={handleClose}>
        <DialogTitle>Subscribe</DialogTitle>
        <DialogContent>
          <DialogContentText>
            {description}
          </DialogContentText>
          <TextField
            autoFocus
            margin="dense"
            label="Port Address"
            fullWidth
            value={addr}
            variant="outlined"
            onChange={(e: any)=>setAddr(e.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={handleClose}>Cancel</Button>
          <Button onClick={()=>{handleCommit(addr); handleClose()}}>{btn}</Button>
        </DialogActions>
      </Dialog>
    </div>
  );
}
