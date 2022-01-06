import { IDefine } from '../../interfaces';
import { Declaration } from './base';

export class Define extends Declaration implements IDefine {
    public constructor(define: IDefine) {
        super(define);
    }
}
