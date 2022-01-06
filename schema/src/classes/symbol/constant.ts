import { IConstant, Searchable } from '../../interfaces';
import { Declaration } from './base';

export class Constant extends Declaration implements IConstant, Searchable {
    public constructor(constant: IConstant) {
        super(constant);
    }
}
