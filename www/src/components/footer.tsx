import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faLinkedin, faGithub, faTwitter} from "@fortawesome/free-brands-svg-icons";
import { Container, Box, Paper } from "@mui/material";
import {ReactComponent as ParthLogo} from "../images/parth.svg"

function Footer (props: {dark: boolean}) {
    const {dark} = props;

    return (
        <Container component={Paper} elevation={0} sx={{width:"50%"}}>
            <Box display="flex" alignContent="center" justifyContent="space-between">
                <Box order={1} display="flex">
                    <Box marginY="auto" component="div">
                        Made with <span style={{color: "red"}}>♥</span> by
                    </Box>
                    <Box>
                        <a href="https://parthpant.github.io" style={{textDecoration: "none", color:"inherit"}}>
                            <ParthLogo height="6em" width="auto" style={{padding:0, margin:0}} fill={dark?"white":"black"}/>
                        </a>
                    </Box>
                </Box>

                {/* <Divider orientation="vertical" flexItem variant="middle"/> */}

                <Box order={3} display="flex">
                    <Box marginX={1} component="div" marginY="auto">
                        <a href="https://github.com/ParthPant/PP8085" style={{textDecoration: "none", color:"inherit"}}>
                        <FontAwesomeIcon icon={faGithub} size="2x"/>
                        </a>
                    </Box>
                    <Box marginX={1} component="div" marginY="auto">
                        <a href="https://www.linkedin.com/in/parth-pant-866bb4189/" style={{textDecoration: "none", color:"inherit"}}>
                        <FontAwesomeIcon icon={faLinkedin} size="2x"/>
                        </a>
                    </Box>
                    <Box marginX={1} component="div" marginY="auto">
                        <a href="https://twitter.com/PantParth" style={{textDecoration: "none", color:"inherit"}}>
                        <FontAwesomeIcon icon={faTwitter} size="2x"/>
                        </a>
                    </Box>
               </Box>
            </Box>
        </Container>
    )
}

export default Footer;