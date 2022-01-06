import { IDefine, Searchable } from '../../interfaces';
import { Declaration } from './base';

export class Define extends Declaration implements IDefine, Searchable {
    public constructor(define: IDefine) {
        super(define);
    }
}
