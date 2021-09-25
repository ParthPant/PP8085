import { PP8085 } from 'pp8085';

function Status(props: any) {
    if (props.cpu) {
        return (
            <table>
                <tr>
                    <td>{props.cpu.get_a()}</td>
                    <td>{props.cpu.get_f()}</td>
                </tr>
                <tr>
                    <td>{props.cpu.get_b()}</td>
                    <td>{props.cpu.get_c()}</td>
                </tr>
                <tr>
                    <td>{props.cpu.get_d()}</td>
                    <td>{props.cpu.get_e()}</td>
                </tr>
                <tr>
                    <td>{props.cpu.get_h()}</td>
                    <td>{props.cpu.get_l()}</td>
                </tr>
                <tr>
                    <td>{props.cpu.get_sp()}</td>
                    <td>{props.cpu.get_pc()}</td>
                </tr>
            </table>
        )
    } else {
        return (
            <p>loading</p>
        )
    }
}

export default Status;