import { IDefine, Searchable, Identifier } from '../../interfaces';
import { Declaration } from './base';

export class Define extends Declaration implements IDefine, Searchable {
    readonly identifier: Identifier = Identifier.Define;
    
    public constructor(define: IDefine) {
        super(define);
    }
}
