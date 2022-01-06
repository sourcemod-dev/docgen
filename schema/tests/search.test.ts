import axios from 'axios';
import { Bundle } from '../src';

test('test', async () => {
    const bundle: Bundle = await axios.get('https://raw.githubusercontent.com/sourcemod-dev/manifest/bundles/core.bundle')

    console.log('wtf');
})
