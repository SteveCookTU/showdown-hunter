import { embed, user as userClient, org as orgClient } from 'https://npm.tfl.dev/@trufflehq/sdk';

let subscription;

export function set_callback(c) {
    subscription = userClient.observable.subscribe({
        next: (u) => {
            c(u.id.toString());
        },
        error: (error) => {
            console.error(error);
        },
        complete: () => {}
    });
}

export function maximize() {
    embed.setStyles({
        height: "725px",
        width: "725px"
    })
}

export function minimize() {
    embed.setStyles({
        height: "40px",
        width: "100px"
    })
}
