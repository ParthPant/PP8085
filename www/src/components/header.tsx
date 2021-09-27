import { Box } from "@mui/system";
import {ReactComponent as Logo} from "../images/8085.svg"

function Header () {
    return (
        <Box sx={{p:0, m:0}} display="flex" justifyContent="start">
            <Box marginY="auto" display="flex" flexDirection="column">
                <Logo height="10em" width="auto"/>
            </Box>
            <Box marginY="auto" component="h3">Intel 8085<br/>MICROPROCESSOR EMULATOR</Box>
        </Box>
    )
}

export default Header;