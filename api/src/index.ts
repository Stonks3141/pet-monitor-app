import express from 'express';
import authRouter from './auth';

const app = express();
const port = 8080;

app.use(express.static('../client/dist'));
app.use('/', authRouter);

app.listen(port, () => {
    console.log(`App listening on port ${port}`);
});

export {};
