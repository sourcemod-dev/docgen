import { IConstant, Searchable, Identifier } from '../../interfaces';
import { Declaration } from './base';

export class Constant extends Declaration implements IConstant, Searchable {
    readonly identifier: Identifier = Identifier.Constant;
    
    public constructor(constant: IConstant) {
        super(constant);
    }
}
