import {Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, Button} from '@mui/material'

function WarnDialog (props:{heading: string, description: string, open: boolean, handleClose: ()=>void}) {
    let {heading, description, open, handleClose} = props;
    
    return (
        <Dialog
        open={open}
        onClose={handleClose}
        aria-labelledby="alert-dialog-title"
        aria-describedby="alert-dialog-description">
            <DialogTitle id="alert-dialog-title">
                {heading}
            </DialogTitle>
            <DialogContent>
                <DialogContentText id="alert-dialog-description">
                    {description}
                </DialogContentText>
            </DialogContent>
            <DialogActions>
                <Button onClick={handleClose}>Close</Button>
            </DialogActions>
      </Dialog>
    )
}

export default WarnDialog;