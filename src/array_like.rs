pub fn fill_arr_u8(input: &[u8], length: usize, truncate: bool) -> Vec<u8> {
    let bytes = input.to_vec();
    let mut pwd_bytes_res: Vec<u8>;
    if bytes.len() < length {
        pwd_bytes_res = bytes;
        while pwd_bytes_res.len() < length {
            pwd_bytes_res.push(0);
        }
    } else if bytes.len() > length {
        if truncate {
            pwd_bytes_res = bytes[0..length].to_vec();
        } else {
            pwd_bytes_res = input.to_vec()
        }
    } else {
        pwd_bytes_res = bytes
    }

    return pwd_bytes_res;
}

pub fn get_vec_max<T: std::cmp::PartialOrd>(list: &Vec<T>) -> &T {
    let mut max_number = &list[0];
    for elem in list {
        if elem > max_number {
            max_number = &elem;
        }
    }
    return max_number;
}

pub fn get_vec_min<T: std::cmp::PartialOrd>(list: &Vec<T>) -> &T {
    let mut min_number = &list[0];
    for elem in list {
        if elem < min_number {
            min_number = &elem;
        }
    }
    return min_number;
}
